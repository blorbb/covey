pub mod config;
mod input;
mod list_item;
pub mod plugin;
mod result_list;
mod spawn;
pub mod hotkey;

use std::{future::Future, path::PathBuf, sync::LazyLock};

use color_eyre::eyre::{bail, Context, Result};
use hotkey::Hotkey;
pub use input::Input;
pub use list_item::ListItem;
use plugin::{Action, Plugin, PluginEvent};
pub use result_list::ResultList;
use tracing::info;

pub static CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    dirs::config_dir()
        .expect("config dir must exist")
        .join("qpmu")
});
pub static PLUGINS_DIR: LazyLock<PathBuf> = LazyLock::new(|| CONFIG_DIR.join("plugins"));
pub static CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| CONFIG_DIR.join("config.toml"));
pub static DATA_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| dirs::data_dir().expect("data dir must exist").join("qpmu"));

/// Main public API for interacting with qpmu.
///
/// The input string and results list may be out of sync.
#[derive(Debug)]
pub struct Model {
    input: Input,
    results: ResultList,
    plugins: Vec<Plugin>,
    dispatched_actions: u64,
    activated_actions: u64,
}

impl Model {
    pub fn new(plugins: Vec<Plugin>) -> Self {
        Self {
            input: Input::default(),
            results: ResultList::default(),
            plugins,
            dispatched_actions: 0,
            activated_actions: 0,
        }
    }

    pub fn input(&self) -> &Input {
        &self.input
    }

    pub fn results(&self) -> &ResultList {
        &self.results
    }

    pub fn set_list_selection(&mut self, selection: usize, fe: &mut impl Frontend) {
        self.results.set_selection(selection);
        fe.set_list_selection(self.results.selection());
    }

    pub fn move_list_selection(&mut self, delta: isize, fe: &mut impl Frontend) {
        self.results.move_selection_signed(delta);
        fe.set_list_selection(self.results.selection());
    }

    pub fn activate(&mut self) -> Option<impl Future<Output = Result<PluginEvent>> + Send + use<>> {
        let item = self.results.selected_item().cloned()?;
        let query = self.input.contents.clone();

        Some(async move { Ok(PluginEvent::Run(item.activate(query).await?)) })
    }

    pub fn alt_activate(
        &mut self,
    ) -> Option<impl Future<Output = Result<PluginEvent>> + Send + use<>> {
        let item = self.results.selected_item().cloned()?;
        let query = self.input.contents.clone();

        Some(async move { Ok(PluginEvent::Run(item.alt_activate(query).await?)) })
    }

    pub fn hotkey_activate(
        &mut self,
        hotkey: Hotkey,
    ) -> Option<impl Future<Output = Result<PluginEvent>> + Send + use<>> {
        let item = self.results.selected_item().cloned()?;
        let query = self.input.contents.clone();

        Some(async move { Ok(PluginEvent::Run(item.hotkey_activate(query, hotkey).await?)) })
    }

    pub fn complete(&mut self) -> Option<impl Future<Output = Result<PluginEvent>> + Send + use<>> {
        let item = self.results.selected_item().cloned()?;
        let query = self.input.contents.clone();

        Some(async move {
            if let Some(new) = item.complete(query).await? {
                Ok(PluginEvent::Run(vec![Action::SetInput(new)]))
            } else {
                // do nothing
                Ok(PluginEvent::Run(vec![]))
            }
        })
    }

    /// Sets the input string and returns a future that should be passed back
    /// into the model later.
    ///
    /// This function should generally **not** be awaited.
    pub fn set_input(
        &mut self,
        input: Input,
    ) -> impl Future<Output = Result<PluginEvent>> + Send + use<> {
        self.input = input.clone();
        self.dispatched_actions += 1;

        let plugins = self.plugins.clone();
        let actioni = self.dispatched_actions;
        async move {
            for plugin in plugins {
                let Some(stripped) = input.contents.strip_prefix(plugin.prefix()) else {
                    continue;
                };
                let list = plugin.query(stripped).await?;

                return Ok(PluginEvent::SetList {
                    list,
                    index: actioni,
                });
            }

            bail!("no plugin activated")
        }
    }

    /// All of these should run very quickly, so it's fine to run on the main thread.
    #[must_use = "if this returns true you must call `set_input`"]
    #[tracing::instrument(skip_all)]
    pub fn handle_event(&mut self, event: Result<PluginEvent>, fe: &mut impl Frontend) -> bool {
        let event = match event {
            Ok(ev) => ev,
            Err(e) => {
                fe.display_error("Error in plugin", e);
                return false;
            }
        };

        match event {
            PluginEvent::SetList { list, index } => {
                if index <= self.activated_actions {
                    return false;
                }
                self.activated_actions = index;
                self.results.reset(list);
                fe.set_list(&self.results);
                false
            }
            PluginEvent::Run(actions) => {
                let mut should_reset_input = false;
                for action in actions {
                    should_reset_input = self.handle_action(action, fe);
                }
                should_reset_input
            }
        }
    }

    /// Returns whether a [`set_input`] future should be made after this.
    #[tracing::instrument(skip_all)]
    fn handle_action(&mut self, event: Action, fe: &mut impl Frontend) -> bool {
        info!("handling action {event:?}");
        match event {
            Action::Close => fe.close(),
            Action::RunCommand(cmd, args) => {
                if let Err(e) = spawn::free_null(&cmd, &args).context(format!(
                    "failed to run command `{cmd} {args}`",
                    args = args.join(" ")
                )) {
                    fe.display_error("Error running command", e);
                }
            }
            Action::RunShell(str) => {
                if let Err(e) = spawn::free_null("sh", ["-c", &str])
                    .context(format!("failed to run command `{str}`"))
                {
                    fe.display_error("Error running command", e);
                }
            }
            Action::Copy(str) => {
                fe.copy(str);
            }
            Action::SetInput(input) => {
                self.input = input.clone();
                fe.set_input(input);
                return true;
            }
        }
        false
    }
}

pub trait Frontend {
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
    fn set_list(&mut self, list: &ResultList);

    fn set_list_selection(&mut self, index: usize);

    fn display_error(&mut self, title: &str, error: color_eyre::eyre::Report);
}
