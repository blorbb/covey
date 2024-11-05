use std::sync::mpsc;

use color_eyre::eyre::Result;
use eframe::{
    egui::{self, Button, CentralPanel, Key, Modifiers, TextEdit, Ui, Vec2, Widget},
    CreationContext,
};

use crate::plugins::{self, Plugin, PluginEvent, UiEvent};

#[derive(Debug)]
pub struct App {
    query: String,
    results: Vec<String>,
    selection: usize,
    ui_events: &'static mpsc::Sender<UiEvent>,
    plugin_events: &'static mpsc::Receiver<PluginEvent>,
}

impl App {
    /// Renders the application.
    ///
    /// This should only be called once.
    pub fn run(plugins: Vec<Plugin>) -> Result<()> {
        use color_eyre::eyre::bail;

        let (ui_events, plugin_events) = plugins::comm::create_channel(plugins);
        let (ui_events, plugin_events) = (
            Box::leak(Box::new(ui_events)),
            Box::leak(Box::new(plugin_events)),
        );

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
            Box::new(|cc| {
                // increase font size for everything
                let ctx = &cc.egui_ctx;
                ctx.all_styles_mut(|style| {
                    style
                        .text_styles
                        .iter_mut()
                        .for_each(|(_, text)| text.size *= 1.6)
                });

                Ok(Box::new(Self::new(cc, ui_events, plugin_events)))
            }),
        ) {
            bail!("{e}");
        };

        Ok(())
    }

    pub fn new(
        _cc: &CreationContext<'_>,
        ui_events: &'static mpsc::Sender<UiEvent>,
        plugin_events: &'static mpsc::Receiver<PluginEvent>,
    ) -> Self {
        Self {
            query: Default::default(),
            results: Default::default(),
            selection: Default::default(),
            ui_events,
            plugin_events,
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
                for (i, item) in self.results.iter().enumerate() {
                    ui.add(Row::new(&mut self.selection, i, item, ""));
                }
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

struct Row<'sel, Value> {
    title: String,
    description: String,
    current_value: &'sel mut Value,
    selected_value: Value,
}

impl<'sel, Value: PartialEq> Row<'sel, Value> {
    pub fn new(
        current_value: &'sel mut Value,
        selected_value: Value,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            current_value,
            selected_value,
        }
    }
}

impl<Value: PartialEq> Widget for Row<'_, Value> {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let fill = if *self.current_value == self.selected_value {
            ui.visuals().selection.bg_fill
        } else {
            ui.visuals().panel_fill
        };

        let button = Button::new(self.title)
            .frame(false)
            .min_size(Vec2::new(ui.available_width(), 0.0))
            .fill(fill)
            .ui(ui);

        if button.clicked() {
            *self.current_value = self.selected_value;
        }

        button
    }
}
