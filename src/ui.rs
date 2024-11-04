use std::sync::mpsc;

use color_eyre::eyre::Result;
use eframe::{
    egui::{self, CentralPanel, Key, Modifiers, TextEdit, Ui},
    CreationContext,
};

use crate::plugins::{self, Plugin, PluginEvent, UiEvent};

#[derive(Debug)]
pub struct App {
    query: String,
    results: Vec<String>,
    selection: usize,
    ui_events: mpsc::Sender<UiEvent>,
    plugin_events: mpsc::Receiver<PluginEvent>,
}

impl App {
    /// Renders the application.
    ///
    /// This should only be called once.
    pub fn run(plugins: Vec<Plugin>) -> Result<()> {
        use color_eyre::eyre::bail;

        let native_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_max_inner_size([800.0, 600.0])
                .with_resizable(false)
                .with_decorations(false)
                .with_active(true)
                .with_transparent(true)
                .with_always_on_top(),
            centered: true,
            ..Default::default()
        };

        if let Err(e) = eframe::run_native(
            "qpmu",
            native_options,
            Box::new(|cc| Ok(Box::new(Self::new(cc, plugins)))),
        ) {
            bail!("{e}");
        };

        Ok(())
    }

    pub fn new(_cc: &CreationContext<'_>, plugins: Vec<Plugin>) -> Self {
        let (sender, receiver) = plugins::comm::create_channel(plugins);

        Self {
            query: Default::default(),
            results: Default::default(),
            selection: Default::default(),
            ui_events: sender,
            plugin_events: receiver,
        }
    }

    fn apply_plugin_event(&mut self, ev: PluginEvent) {
        match ev {
            PluginEvent::SetList(vec) => {
                self.results = vec.into_iter().map(|e| e.title).collect();
                self.selection = 0;
            }
            PluginEvent::Activate(action) => todo!(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(ev) = self.plugin_events.try_recv() {
            self.apply_plugin_event(ev);
        }

        CentralPanel::default()
            .frame(egui::Frame::none().outer_margin(10.0))
            .show(ctx, |ui| {
                // handle special keys first, do not pass them through
                if consume_input(ui, Key::ArrowDown) {
                    self.selection += 1;
                } else if consume_input(ui, Key::ArrowUp) {
                    self.selection -= 1;
                }

                let text_edit = TextEdit::singleline(&mut self.query)
                    .hint_text("Search...")
                    .desired_width(f32::INFINITY)
                    .return_key(None)
                    .show(ui);

                if !text_edit.response.has_focus() {
                    text_edit.response.request_focus();
                }

                if text_edit.response.changed() {
                    self.ui_events
                        .send(UiEvent {
                            query: self.query.clone(),
                        })
                        .unwrap();
                }
                // TODO: focus flickering if a value is clicked
                // TODO: style this like a list
                for (i, item) in self.results.iter().enumerate() {
                    ui.radio_value(&mut self.selection, i, item);
                }

                ctx
            });
    }

    // necessary to make the window completely transparent
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0; 4]
    }
}

pub fn consume_input(ui: &mut Ui, key: Key) -> bool {
    ui.input_mut(|state| state.consume_key(Modifiers::NONE, key))
}
