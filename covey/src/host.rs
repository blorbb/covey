use std::{
    fs,
    future::Future,
    io::{Read as _, Write as _},
    sync::Arc,
};

use color_eyre::eyre::{Context, Result, bail, eyre};
use covey_config::{
    config::GlobalConfig,
    keyed_list::{Id, KeyedList},
};
use parking_lot::Mutex;
use tracing::{debug, error, info, warn};

use crate::{
    CONFIG_PATH, Frontend, List, Plugin,
    event::{Action, ListItemId, PluginEvent},
};

struct HostInner {
    plugins: KeyedList<Plugin>,
    dispatched_actions: u64,
    activated_actions: u64,
    fe: Box<dyn Frontend>,
    config: GlobalConfig,
}

/// Main public API for interacting with covey.
///
/// When an action is returned from a plugin, the frontend is updated.
///
/// This is cheap to clone.
#[derive(Clone)]
pub struct Host {
    inner: Arc<Mutex<HostInner>>,
}

impl Host {
    pub fn new(fe: impl Frontend) -> Result<Self> {
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
        let plugins = Self::load_plugins(&global_config);

        info!("found plugins: {plugins:?}");

        Ok(Self {
            inner: Arc::new(Mutex::new(HostInner {
                plugins,
                dispatched_actions: 0,
                activated_actions: 0,
                fe: Box::new(fe),
                config: global_config,
            })),
        })
    }

    /// Reads the manifests of every plugin listed in the config.
    fn load_plugins(config: &GlobalConfig) -> KeyedList<Plugin> {
        KeyedList::new_lossy(config.plugins.iter().filter_map(|config| {
            match Plugin::new(config.clone()) {
                Ok(plugin) => {
                    debug!("found plugin {plugin:?}");
                    Some(plugin)
                }
                Err(e) => {
                    error!("error finding plugin: {e}");
                    None
                }
            }
        }))
    }

    /// Writes the config to the [`CONFIG_PATH`].
    ///
    /// # Errors
    /// Returns an error if there was an IO or serialization issue.
    fn write_config(config: &GlobalConfig) -> Result<()> {
        // stringify here to avoid truncating the file then erroring
        let toml_str = toml::to_string_pretty(config)?;

        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&*CONFIG_PATH)?;

        file.write_all(toml_str.as_bytes())?;

        Ok(())
    }

    fn make_event_future<Fut>(&self, event: Fut) -> impl Future<Output = ()> + use<Fut>
    where
        Fut: Future<Output = Result<PluginEvent>> + Send + 'static,
    {
        let this = self.clone();
        async move {
            this.handle_event(event.await).await;
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn activate(
        &self,
        item: ListItemId,
        command_name: String,
    ) -> impl Future<Output = ()> + use<> {
        debug!("activating {item:?}");

        self.make_event_future(async move {
            item.plugin
                .activate(item.local_id, command_name)
                .await
                .map(PluginEvent::Run)
        })
    }

    /// Calls a plugin with this input.
    #[tracing::instrument(skip(self))]
    pub fn query(&self, input: String) -> impl Future<Output = ()> + use<> {
        debug!("setting input to {input:?}");
        let (plugins, this_action_index, icon_themes) = {
            let mut inner = self.inner.lock();
            inner.dispatched_actions += 1;

            (
                inner.plugins.clone(),
                inner.dispatched_actions,
                inner.config.app.icon_themes.clone(),
            )
        };

        self.make_event_future(async move {
            for plugin in plugins {
                let Some(stripped) = input.strip_prefix(plugin.prefix()) else {
                    continue;
                };
                debug!("querying plugin {plugin:?}");
                let proto_list = plugin.query(stripped).await?;

                return Ok(PluginEvent::SetList {
                    list: List::from_proto(&plugin, &icon_themes, proto_list),
                    index: this_action_index,
                });
            }

            bail!("no plugin activated")
        })
    }

    async fn handle_event(&self, event: Result<PluginEvent>) {
        let chained_query = self.inner.lock().handle_event(event);

        if let Some(query) = chained_query {
            // indirection needed to avoid infinitely sized future
            Box::pin(self.query(query)).await;
        }
    }

    /// Reloads all plugins with the new configuration.
    ///
    /// This will also call [`Frontend::reload`].
    #[tracing::instrument(skip_all)]
    pub fn reload(&self, config: GlobalConfig) {
        debug!("reloading");
        let mut inner = self.inner.lock();
        inner.plugins = Self::load_plugins(&config);
        // TODO: spawn this in another task and handle errors properly
        Self::write_config(&config).expect("TODO");
        inner.config = config.clone();
        inner.fe.reload(config);
    }

    pub fn reload_plugin(&self, plugin_id: &Id) {
        debug!("reloading plugin {plugin_id:?}");
        let Some(plugin_config) = self
            .inner
            .lock()
            .config
            .plugins
            .get(plugin_id.as_str())
            .cloned()
        else {
            self.inner.lock().fe.display_error(
                "Failed to reload plugin",
                eyre!("could not find plugin's config"),
            );
            return;
        };

        let old_plugins: Vec<_> = self.inner.lock().plugins.iter().cloned().collect();

        let new_plugins = old_plugins.into_iter().filter_map(|plugin| {
            if plugin.id() == plugin_id {
                match Plugin::new(plugin_config.clone()) {
                    Ok(plugin) => Some(plugin),
                    Err(e) => {
                        self.inner
                            .lock()
                            .fe
                            .display_error("Failed to reload plugin", e);
                        None
                    }
                }
            } else {
                Some(plugin)
            }
        });

        self.inner.lock().plugins =
            KeyedList::new(new_plugins).expect("new keyed list should have same keys");
    }

    pub fn config(&self) -> GlobalConfig {
        self.inner.lock().config.clone()
    }

    /// Ordered set of all plugins.
    #[tracing::instrument(skip(self))]
    pub fn plugins(&self) -> KeyedList<Plugin> {
        self.inner.lock().plugins.clone()
    }

    /// Swaps out the frontend used.
    ///
    /// Note: this is only used for hot module reloading support.
    pub fn set_frontend(&self, fe: impl Frontend) {
        self.inner.lock().fe = Box::new(fe);
    }
}

impl HostInner {
    /// Optionally returns another string that should be queried.
    #[tracing::instrument(skip(self))]
    fn handle_event(&mut self, event: Result<PluginEvent>) -> Option<String> {
        debug!("handling event");

        match event {
            Ok(PluginEvent::SetList { list, index }) => {
                if index <= self.activated_actions {
                    return None;
                }
                self.activated_actions = index;
                self.fe.set_list(list);
            }
            Ok(PluginEvent::Run(actions)) => {
                return actions
                    .into_iter()
                    .fold(None, |opt, action| self.handle_action(action).or(opt));
            }
            Err(e) => {
                self.fe.display_error("Error in plugin", e);
            }
        }

        None
    }

    /// Optionally returns another string that should be queried.
    #[tracing::instrument(skip(self))]
    fn handle_action(&mut self, action: Action) -> Option<String> {
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
                return Some(input.contents);
            }
        }
        None
    }
}
