use std::{sync::mpsc, thread};

use super::{Plugin, PluginEvent, UiEvent};

/// Spawns a detached thread that receives events from the
/// ui and sends events from plugins.
///
/// This function should only be called once.
pub fn create_channel(
    plugins: Vec<Plugin>,
) -> (mpsc::Sender<UiEvent>, mpsc::Receiver<PluginEvent>) {
    let (ui_sender, ui_receiver) = mpsc::channel::<UiEvent>();
    let (plugin_sender, plugin_receiver) = mpsc::channel::<PluginEvent>();

    thread::spawn(move || {
        while let Ok(ui_event) = ui_receiver.recv() {
            eprintln!("got ui event");
            if let Some(action) = process_ui_event(&plugins, ui_event) {
                plugin_sender
                    .send(action)
                    .expect("plugin event receiver must not be closed");
            }
        }
    });

    (ui_sender, plugin_receiver)
}

fn process_ui_event(plugins: &[Plugin], ev: UiEvent) -> Option<PluginEvent> {
    plugins
        .iter()
        .find_map(|plugin| plugin.try_call_input(&ev.query))
        .map_or(Some(PluginEvent::SetList(vec![])), |ev| {
            ev.ok().map(PluginEvent::SetList)
        })
}
