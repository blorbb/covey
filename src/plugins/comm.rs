use std::{sync::mpsc, thread};

use color_eyre::eyre::Result;

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

            match process_ui_event(&plugins, ui_event) {
                Ok(action) => plugin_sender
                    .send(action)
                    .expect("plugin event receiver must not be closed"),
                Err(e) => {
                    eprintln!("error processing event: {e}")
                }
            }
        }
    });

    (ui_sender, plugin_receiver)
}

fn process_ui_event(plugins: &[Plugin], ev: UiEvent) -> Result<PluginEvent> {
    Ok(match ev {
        UiEvent::InputChanged { query } => PluginEvent::SetList(
            plugins
                .iter()
                .find_map(|plugin| plugin.try_call_input(&query))
                .transpose()?
                .unwrap_or_default(),
        ),

        UiEvent::Activate { item } => PluginEvent::Activate(item.activate()?),
    })
}
