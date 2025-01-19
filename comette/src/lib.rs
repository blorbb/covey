pub mod config;
mod details;
pub mod hotkey;
mod input;
mod list_item;
pub mod lock;
pub mod plugin;
mod result_list;
mod spawn;

use std::{fmt, future::Future, path::PathBuf, sync::LazyLock};

use color_eyre::eyre::{bail, Context, Result};
use config::Config;
use hotkey::Hotkey;
pub use input::Input;
pub use list_item::{Icon, ListItem, ListItemId};
use lock::SharedMutex;
use plugin::{proto, Action, Plugin, PluginEvent};
pub use result_list::{BoundedUsize, ListStyle, ResultList};
use tokio::sync::mpsc::UnboundedSender;
use tracing::{debug, error, info};

pub static CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    dirs::config_dir()
        .expect("config dir must exist")
        .join("comette")
});
pub static CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| CONFIG_DIR.join("config.toml"));
pub static DATA_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| dirs::data_dir().expect("data dir must exist").join("comette"));

/// Main public API for interacting with comette.
///
/// When an action is returned from a plugin, the frontend is updated.
#[derive(Clone)]
pub struct Model<F> {
    plugins: Vec<Plugin>,
    dispatched_actions: u64,
    activated_actions: u64,
    sender: UnboundedSender<Result<PluginEvent>>,
    fe: F,
}

impl<F: Frontend> Model<F> {
    pub fn new(plugins: Vec<Plugin>, fe: F) -> SharedMutex<Model<F>> {
        let (send, mut recv) = tokio::sync::mpsc::unbounded_channel::<Result<PluginEvent>>();
        let this = Self {
            plugins,
            dispatched_actions: 0,
            activated_actions: 0,
            sender: send,
            fe,
        };

        let this = SharedMutex::new(this);

        tokio::spawn({
            let this = SharedMutex::clone(&this);
            async move {
                while let Some(a) = recv.recv().await {
                    this.lock().handle_event(a);
                }
            }
        });

        this
    }

    fn send_event(&mut self, event: impl Future<Output = Result<PluginEvent>> + Send + 'static) {
        let sender = self.sender.clone();
        tokio::spawn(async move { _ = sender.send(event.await) });
    }

    #[tracing::instrument(skip_all)]
    pub fn activate(&mut self, item: ListItemId) {
        debug!("activating {item:?}");

        self.send_event(async move { item.plugin.activate(item.local_id).await.map(PluginEvent::Run) });
    }

    #[tracing::instrument(skip_all)]
    pub fn alt_activate(&mut self, item: ListItemId) {
        debug!("alt-activating {item:?}");

        self.send_event(async move {
            item.plugin
                .alt_activate(item.local_id)
                .await
                .map(PluginEvent::Run)
        });
    }

    #[tracing::instrument(skip_all)]
    pub fn hotkey_activate(&mut self, item: ListItemId, hotkey: Hotkey) {
        debug!("hotkey-activating {item:?}");

        self.send_event(async move {
            item.plugin
                .hotkey_activate(item.local_id, proto::Hotkey::from(hotkey))
                .await
                .map(PluginEvent::Run)
        });
    }

    #[tracing::instrument(skip_all)]
    pub fn complete(&mut self, item: ListItemId) {
        debug!("completing {item:?}");

        self.send_event(async move {
            if let Some(new) = item.plugin.complete(item.local_id).await? {
                Ok(PluginEvent::Run(vec![Action::SetInput(new)]))
            } else {
                // do nothing
                Ok(PluginEvent::Run(vec![]))
            }
        });
    }

    /// Calls a plugin with this input.
    #[tracing::instrument(skip_all)]
    pub fn query(&mut self, input: String) {
        debug!("setting input to {input:?}");
        self.dispatched_actions += 1;

        let plugins = self.plugins.clone();
        let actioni = self.dispatched_actions;
        self.send_event(async move {
            for plugin in plugins {
                let Some(stripped) = input.strip_prefix(plugin.prefix()) else {
                    continue;
                };
                debug!("querying plugin {plugin:?}");
                let list = plugin.query(stripped).await?;

                return Ok(PluginEvent::SetList {
                    list,
                    index: actioni,
                });
            }

            bail!("no plugin activated")
        })
    }

    #[tracing::instrument(skip_all)]
    fn handle_event(&mut self, event: Result<PluginEvent>) {
        debug!("handling event");
        let event = match event {
            Ok(ev) => ev,
            Err(e) => {
                self.fe.display_error("Error in plugin", e);
                return;
            }
        };

        match event {
            PluginEvent::SetList { list, index } => {
                if index <= self.activated_actions {
                    return;
                }
                self.activated_actions = index;
                self.fe.set_list(list);
            }
            PluginEvent::Run(actions) => {
                actions
                    .into_iter()
                    .for_each(|action| self.handle_action(action));
            }
        }
    }

    #[tracing::instrument(skip_all)]
    fn handle_action(&mut self, action: Action) {
        info!("handling action {action:?}");
        match action {
            Action::Close => self.fe.close(),
            Action::RunCommand(cmd, args) => {
                if let Err(e) = spawn::free_null(&cmd, &args).context(format!(
                    "failed to run command `{cmd} {args}`",
                    args = args.join(" ")
                )) {
                    error!("Error running command: {e:#}");
                    self.fe.display_error("Error running command", e);
                }
            }
            Action::RunShell(str) => {
                if let Err(e) = spawn::free_null("sh", ["-c", &str])
                    .context(format!("failed to run command `{str}`"))
                {
                    error!("Error running command: {e:#}");
                    self.fe.display_error("Error running command", e);
                }
            }
            Action::Copy(str) => {
                self.fe.copy(str);
            }
            Action::SetInput(input) => {
                self.fe.set_input(input.clone());
                self.query(input.contents);
            }
        }
    }

    /// Reloads all plugins with the new configuration.
    #[tracing::instrument(skip_all)]
    pub fn reload(&mut self, config: Config) -> Result<()> {
        debug!("reloading");
        self.plugins = config.load();
        Ok(())
    }

    /// List of all plugins.
    pub fn plugins(&self) -> &[Plugin] {
        &self.plugins
    }
}

impl<F> fmt::Debug for Model<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Model")
            .field("plugins", &self.plugins)
            .field("dispatched_actions", &self.dispatched_actions)
            .field("activated_actions", &self.activated_actions)
            .field("sender", &self.sender)
            .finish()
    }
}

/// A controller for the UI.
///
/// These methods may not be called on the main thread. Many UI
/// frameworks require updates to be called on the main thread,
/// so you will likely need to use channels to communicate these
/// messages.
///
/// None of methods should result in calling methods on [`Model`]
/// which have a `&mut self` receiver (as most of these would call
/// the front end to update again), otherwise an infinite loop may
/// occur. These methods will be called after the model has been
/// updated, so reading from the model is okay.
pub trait Frontend: Send + 'static {
    /// Close the window.
    fn close(&mut self);

    /// Copy a string to the clipboard.
    fn copy(&mut self, str: String);

    /// Set the UI input to the provided input.
    ///
    /// The model will already have an updated input, so do not try to change
    /// the model here. Only modify the front end.
    fn set_input(&mut self, input: Input);

    /// Set the UI results list to the provided list.
    ///
    /// The model will already have an updated list, so do not try to change
    /// the model here. Only modify the front end.
    fn set_list(&mut self, list: ResultList);

    // TODO: refactor this lib to have a custom error type
    fn display_error(&mut self, title: &str, error: color_eyre::eyre::Report);
}
