pub mod config;
mod details;
pub mod hotkey;
mod input;
mod list_item;
pub mod lock;
pub mod plugin;
mod result_list;
mod spawn;

use std::{future::Future, path::PathBuf, pin::Pin, sync::LazyLock};

use color_eyre::eyre::{bail, Context, Result};
use config::Config;
use hotkey::Hotkey;
pub use input::Input;
pub use list_item::{Icon, ListItem};
use lock::SharedMutex;
use plugin::{Action, Plugin, PluginEvent};
pub use result_list::{BoundedUsize, ListStyle, ResultList};
use tokio::sync::mpsc::UnboundedSender;
use tracing::{debug, info};

pub static CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    dirs::config_dir()
        .expect("config dir must exist")
        .join("qpmu")
});
pub static CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| CONFIG_DIR.join("config.toml"));
pub static DATA_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| dirs::data_dir().expect("data dir must exist").join("qpmu"));

/// Main public API for interacting with qpmu.
///
/// Most methods here also call the front end to react to changes.
/// Methods should only be called when a change has happened. They
/// should not be called when updating the UI to match the current
/// state.
///
/// The input string and results list may be out of sync.
pub struct Model<F> {
    input: Input,
    results: ResultList,
    plugins: Vec<Plugin>,
    dispatched_actions: u64,
    activated_actions: u64,
    // TODO: is this actually necessary?
    task_spawner: Box<dyn FnMut(Pin<Box<dyn Future<Output = ()> + Send + 'static>>) + Send>,
    sender: UnboundedSender<Result<PluginEvent>>,
    fe: F,
}

impl<F: Frontend> Model<F> {
    pub fn new(plugins: Vec<Plugin>, fe: F) -> SharedMutex<Model<F>> {
        let (send, mut recv) = tokio::sync::mpsc::unbounded_channel::<Result<PluginEvent>>();
        let this = Self {
            input: Input::default(),
            results: ResultList::default(),
            plugins,
            dispatched_actions: 0,
            activated_actions: 0,
            task_spawner: Box::new(|fut| _ = tokio::spawn(fut)),
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

    pub fn input(&self) -> &Input {
        &self.input
    }

    pub fn results(&self) -> &ResultList {
        &self.results
    }

    #[tracing::instrument(skip_all)]
    pub fn set_list_selection(&mut self, selection: usize) {
        debug!("set list selection to {selection}");
        self.results.set_selection(selection);
        self.fe.set_list_selection(self.results.selection());
    }

    #[tracing::instrument(skip_all)]
    pub fn move_list_selection(&mut self, delta: isize) {
        debug!("moving list selection by {delta}");
        self.results.move_selection_signed(delta);
        self.fe.set_list_selection(self.results.selection());
    }

    fn send_event(&mut self, event: impl Future<Output = Result<PluginEvent>> + Send + 'static) {
        let sender = self.sender.clone();
        (self.task_spawner)(Box::pin(async move { _ = sender.send(event.await) }))
    }

    #[tracing::instrument(skip_all)]
    pub fn activate(&mut self) {
        debug!("activating current selection");
        let Some(item) = self.results.selected_item().cloned() else {
            return;
        };
        debug!("current selection is {item:?}");

        self.send_event(async move { item.activate().await.map(PluginEvent::Run) });
    }

    #[tracing::instrument(skip_all)]
    pub fn alt_activate(&mut self) {
        debug!("alt-activating current selection");
        let Some(item) = self.results.selected_item().cloned() else {
            return;
        };
        debug!("current selection is {item:?}");

        self.send_event(async move { item.alt_activate().await.map(PluginEvent::Run) });
    }

    #[tracing::instrument(skip_all)]
    pub fn hotkey_activate(&mut self, hotkey: Hotkey) {
        debug!("hotkey-activating current selection");
        let Some(item) = self.results.selected_item().cloned() else {
            return;
        };
        debug!("current selection is {item:?}");

        self.send_event(async move { item.hotkey_activate(hotkey).await.map(PluginEvent::Run) });
    }

    #[tracing::instrument(skip_all)]
    pub fn complete(&mut self) {
        debug!("completing current selection");
        let Some(item) = self.results.selected_item().cloned() else {
            return;
        };
        debug!("current selection is {item:?}");

        self.send_event(async move {
            if let Some(new) = item.complete().await? {
                Ok(PluginEvent::Run(vec![Action::SetInput(new)]))
            } else {
                // do nothing
                Ok(PluginEvent::Run(vec![]))
            }
        });
    }

    /// Sets the input string and calls a plugin with this input.
    ///
    /// This method calls [`Frontend::set_input`].
    #[tracing::instrument(skip_all)]
    pub fn set_input(&mut self, input: Input) {
        debug!("setting input to {input:?}");
        self.input = input.clone();
        self.dispatched_actions += 1;
        self.fe.set_input(&input);

        let plugins = self.plugins.clone();
        let actioni = self.dispatched_actions;
        self.send_event(async move {
            for plugin in plugins {
                let Some(stripped) = input.contents.strip_prefix(plugin.prefix()) else {
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
                self.results = list;
                self.fe.set_list(&self.results);
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
                    self.fe.display_error("Error running command", e);
                }
            }
            Action::RunShell(str) => {
                if let Err(e) = spawn::free_null("sh", ["-c", &str])
                    .context(format!("failed to run command `{str}`"))
                {
                    self.fe.display_error("Error running command", e);
                }
            }
            Action::Copy(str) => {
                self.fe.copy(str);
            }
            Action::SetInput(input) => {
                self.set_input(input);
            }
        }
    }

    /// Refreshes qpmu by re-reading the config.toml file.
    #[tracing::instrument(skip_all)]
    pub fn reload(&mut self) -> Result<()> {
        debug!("reloading");
        let plugins = Config::load_plugins()?;
        self.plugins = plugins;
        Ok(())
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
    fn set_input(&mut self, input: &Input);

    /// Set the UI results list to the provided list.
    ///
    /// The model will already have an updated list, so do not try to change
    /// the model here. Only modify the front end.
    fn set_list(&mut self, list: &ResultList);

    fn set_list_selection(&mut self, index: usize);

    fn display_error(&mut self, title: &str, error: color_eyre::eyre::Report);
}
