use std::{future::Future, path::PathBuf, sync::LazyLock};

use az::SaturatingAs;
use color_eyre::eyre::{bail, Result};
use plugin::{event::PluginEvent, Action, ListItem, Plugin};

pub mod config;
pub mod plugin;
mod spawn;

pub static CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    dirs::config_dir()
        .expect("config dir must exist")
        .join("qpmu")
});
pub static PLUGINS_DIR: LazyLock<PathBuf> = LazyLock::new(|| CONFIG_DIR.join("plugins"));

/// Gets the path to the plugin binary.
fn plugin_file_of(plugin_name: &str) -> PathBuf {
    PLUGINS_DIR.join(plugin_name)
}

#[derive(Debug, Clone, Default)]
pub struct Input {
    pub contents: String,
    /// Range in terms of chars, not bytes
    pub selection: (u16, u16),
}

impl Input {
    pub fn new(contents: String, selection: (u16, u16)) -> Self {
        Self {
            contents,
            selection,
        }
    }

    pub(crate) fn prefix_with(&mut self, prefix: &str) {
        self.contents.insert_str(0, prefix);
        let prefix_len =
            u16::try_from(prefix.chars().count()).expect("prefix should not be insanely long");

        let (a, b) = self.selection;
        self.selection = (a.saturating_add(prefix_len), b.saturating_add(prefix_len));
    }

    pub(crate) fn from_proto(plugin: Plugin, il: plugin::proto::Input) -> Self {
        let mut input = Self {
            contents: il.query,
            selection: (il.range_lb.saturating_as(), il.range_ub.saturating_as()),
        };
        input.prefix_with(&plugin.prefix());
        input
    }
}

#[derive(Debug, Default)]
pub struct ResultList {
    list: Vec<ListItem>,
    selection: usize,
}

impl ResultList {
    pub fn reset(&mut self, list: Vec<ListItem>) {
        self.list = list;
        self.selection = 0;
    }

    pub fn list(&self) -> &[ListItem] {
        &self.list
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    pub fn selection(&self) -> usize {
        self.selection
    }

    pub fn selected_item(&self) -> Option<&ListItem> {
        self.list.get(self.selection)
    }
}

/// Main public API for interacting with qpmu.
///
/// The input string and results list may be out of sync.
#[derive(Debug)]
pub struct Model {
    input: Input,
    results: ResultList,
    plugins: &'static [Plugin],
    dispatched_actions: u64,
    activated_actions: u64,
}

impl Model {
    pub fn new(plugins: &'static [Plugin]) -> Self {
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
        self.results.selection = selection;
        fe.set_list_selection(self.results.selection);
    }

    pub fn move_list_selection(&mut self, delta: isize, fe: &mut impl Frontend) {
        self.results.selection = self.results.selection.saturating_add_signed(delta);
        fe.set_list_selection(self.results.selection);
    }

    pub fn activate(&mut self) -> impl Future<Output = Result<PluginEvent>> + Send + use<> {
        let Some(item) = self.results.selected_item().cloned() else {
            todo!()
        };

        async move { Ok(PluginEvent::Run(item.activate().await?)) }
    }

    pub fn complete(&mut self) -> impl Future<Output = Result<PluginEvent>> + Send + use<> {
        let Some(item) = self.results.selected_item().cloned() else {
            todo!()
        };

        let query = self.input.contents.clone();

        async move {
            if let Some(new) = item.complete(&query).await? {
                Ok(PluginEvent::Run(vec![Action::SetInput(new)]))
            } else {
                Ok(PluginEvent::Run(vec![]))
            }
        }
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

        let plugins = self.plugins;
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
    pub fn handle_event(&mut self, event: Result<PluginEvent>, fe: &mut impl Frontend) -> bool {
        let Ok(event) = event else {
            todo!("set first item to an error message")
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
    fn handle_action(&mut self, event: Action, fe: &mut impl Frontend) -> bool {
        match event {
            Action::Close => fe.close(),
            Action::RunCommand(cmd, args) => {
                spawn::free_null(cmd, args).expect("TODO");
            }
            Action::RunShell(str) => {
                spawn::free_null("sh", ["-c", &str]).expect("TODO");
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
}
