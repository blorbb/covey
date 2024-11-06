use std::sync::mpsc;

use crate::plugins::{self, ListItem, Plugin, PluginActivationAction, PluginEvent, UiEvent};

#[derive(Debug)]
#[tracker::track]
pub struct Launcher {
    pub query: String,
    #[no_eq]
    pub results: Vec<ListItem>,
    pub selection: usize,
    #[do_not_track]
    ui_events: &'static mpsc::Sender<UiEvent>,
    #[do_not_track]
    plugin_events: &'static mpsc::Receiver<PluginEvent>,
}

impl Launcher {
    pub fn new(plugins: Vec<Plugin>) -> Self {
        let (ui_events, plugin_events) = plugins::comm::create_channel(plugins);
        let (ui_events, plugin_events) = (
            Box::leak(Box::new(ui_events)),
            Box::leak(Box::new(plugin_events)),
        );

        Self {
            query: Default::default(),
            results: Default::default(),
            selection: Default::default(),
            ui_events,
            plugin_events,
            tracker: 0,
        }
    }

    fn apply_plugin_event(&mut self, ev: PluginEvent) {
        todo!()
        // match ev {
        //     PluginEvent::SetList(vec) => {
        //         self.results = vec;
        //         self.selection = 0;
        //     }
        //     PluginEvent::Activate(evs) => {
        //         evs.into_iter().for_each(|a| self.apply_activation(a, ctx))
        //     }
        // }
    }

    fn apply_activation(&mut self, activation: PluginActivationAction) {
        todo!()
        // match activation {
        //     PluginActivationAction::Close => ctx.send_viewport_cmd(ViewportCommand::Close),
        //     PluginActivationAction::RunCommandString(cmd) => {
        //         if let Err(e) = std::process::Command::new("sh")
        //             .arg("-c")
        //             .arg(&cmd)
        //             .stdout(Stdio::null())
        //             .stderr(Stdio::null())
        //             .spawn()
        //         {
        //             eprintln!("error running command {cmd:?}: {e}")
        //         }
        //     }
        //     PluginActivationAction::RunCommand((cmd, args)) => {
        //         if let Err(e) = std::process::Command::new(&cmd)
        //             .args(&args)
        //             .stdout(Stdio::null())
        //             .stderr(Stdio::null())
        //             .spawn()
        //         {
        //             eprintln!("error running command {cmd} {args:?}: {e}")
        //         }
        //     }
        //     PluginActivationAction::Copy(string) => {
        //         ctx.copy_text(string);
        //     }
        // }
    }

    pub fn ui_events(&self) -> &'static mpsc::Sender<UiEvent> {
        self.ui_events
    }

    pub fn plugin_events(&self) -> &'static mpsc::Receiver<PluginEvent> {
        self.plugin_events
    }
}

#[derive(Debug)]
pub enum LauncherMsg {
    /// Set the query to a string
    Query(String),
    /// Set the results list
    SetList(Vec<ListItem>),
    /// Selects a specific index of the results list
    Select(usize),
    /// Change the selection index by a certain amount
    SelectDelta(isize),
    /// Activate the current selected item
    Activate,
    /// Close (hide) the window
    Close,
}
