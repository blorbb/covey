use core::fmt;
use std::{fs, future::Future, io::Read as _};

use color_eyre::eyre::{bail, Context, Result};
use indexmap::IndexSet;
use tokio::sync::mpsc::UnboundedSender;
use tracing::{debug, error, info};

use crate::{
    config::GlobalConfig,
    event::{Action, ListItemId, PluginEvent},
    hotkey::Hotkey,
    lock::SharedMutex,
    proto, Frontend, Plugin, CONFIG_PATH,
};

// TODO: remove clone

/// Main public API for interacting with comette.
///
/// When an action is returned from a plugin, the frontend is updated.
#[derive(Clone)]
pub struct Host<F> {
    pub(crate) plugins: IndexSet<Plugin>,
    pub(crate) dispatched_actions: u64,
    pub(crate) activated_actions: u64,
    pub(crate) sender: UnboundedSender<Result<PluginEvent>>,
    pub(crate) fe: F,
}

impl<F: Frontend> Host<F> {
    pub fn new(fe: F) -> Result<SharedMutex<Host<F>>> {
        info!("reading config from file: {:?}", &*CONFIG_PATH);

        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .truncate(false)
            .open(&*CONFIG_PATH)?;

        let mut s = String::new();
        file.read_to_string(&mut s)?;

        debug!("read config:\n{s}");

        let global_config: GlobalConfig = toml::from_str(&s)?;
        let plugins = global_config.load_plugins();

        info!("found plugins: {plugins:?}");

        // make a channel to receive events and handle them
        let (send, mut recv) = tokio::sync::mpsc::unbounded_channel::<Result<PluginEvent>>();
        let this = Self {
            plugins,
            dispatched_actions: 0,
            activated_actions: 0,
            sender: send,
            fe,
        };

        // TODO: spawn local instead, avoid the mutex?
        let this = SharedMutex::new(this);

        tokio::spawn({
            let this = SharedMutex::clone(&this);
            async move {
                while let Some(a) = recv.recv().await {
                    this.lock().handle_event(a);
                }
            }
        });

        Ok(this)
    }

    pub(crate) fn send_event(
        &mut self,
        event: impl Future<Output = Result<PluginEvent>> + Send + 'static,
    ) {
        let sender = self.sender.clone();
        tokio::spawn(async move { _ = sender.send(event.await) });
    }

    #[tracing::instrument(skip(self))]
    pub fn activate(&mut self, item: ListItemId) {
        debug!("activating {item:?}");

        self.send_event(async move {
            item.plugin
                .activate(item.local_id)
                .await
                .map(PluginEvent::Run)
        });
    }

    #[tracing::instrument(skip(self))]
    pub fn alt_activate(&mut self, item: ListItemId) {
        debug!("alt-activating {item:?}");

        self.send_event(async move {
            item.plugin
                .alt_activate(item.local_id)
                .await
                .map(PluginEvent::Run)
        });
    }

    #[tracing::instrument(skip(self))]
    pub fn hotkey_activate(&mut self, item: ListItemId, hotkey: Hotkey) {
        debug!("hotkey-activating {item:?}");

        self.send_event(async move {
            item.plugin
                .hotkey_activate(item.local_id, proto::Hotkey::from(hotkey))
                .await
                .map(PluginEvent::Run)
        });
    }

    #[tracing::instrument(skip(self))]
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
    #[tracing::instrument(skip(self))]
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

    #[tracing::instrument(skip(self))]
    pub(crate) fn handle_event(&mut self, event: Result<PluginEvent>) {
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

    #[tracing::instrument(skip(self))]
    pub(crate) fn handle_action(&mut self, action: Action) {
        info!("handling action {action:?}");
        match action {
            Action::Close => self.fe.close(),
            Action::RunCommand(cmd, args) => {
                if let Err(e) = crate::spawn::free_null(&cmd, &args).context(format!(
                    "failed to run command `{cmd} {args}`",
                    args = args.join(" ")
                )) {
                    error!("Error running command: {e:#}");
                    self.fe.display_error("Error running command", e);
                }
            }
            Action::RunShell(str) => {
                if let Err(e) = crate::spawn::free_null("sh", ["-c", &str])
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
    #[tracing::instrument(skip(self))]
    pub fn reload(&mut self, config: GlobalConfig) -> Result<()> {
        debug!("reloading");
        self.plugins = config.load_plugins();
        Ok(())
    }

    /// Ordered set of all plugins.
    #[tracing::instrument(skip(self))]
    pub fn plugins(&self) -> &IndexSet<Plugin> {
        &self.plugins
    }
}

impl<F> fmt::Debug for Host<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Model")
            .field("plugins", &self.plugins)
            .field("dispatched_actions", &self.dispatched_actions)
            .field("activated_actions", &self.activated_actions)
            .field("sender", &self.sender)
            .finish()
    }
}
