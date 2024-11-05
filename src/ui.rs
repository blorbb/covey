use std::{process::Stdio, sync::mpsc};

use color_eyre::eyre::{bail, Result};
use eframe::{
    egui::{
        self, text::LayoutJob, Button, CentralPanel, Color32, Context, FontData, FontDefinitions,
        Key, Modifiers, ScrollArea, TextEdit, TextFormat, Ui, Vec2, ViewportCommand, Widget,
    },
    CreationContext,
};

use crate::plugins::{self, ListItem, Plugin, PluginActivationAction, PluginEvent, UiEvent};

const WINDOW_WIDTH: f32 = 800.0;
const MAX_WINDOW_HEIGHT: f32 = 600.0;
const PADDING: f32 = 10.0;

#[derive(Debug)]
pub struct App {
    query: String,
    results: Vec<ListItem>,
    selection: usize,
    ui_events: &'static mpsc::Sender<UiEvent>,
    plugin_events: &'static mpsc::Receiver<PluginEvent>,
}

impl App {
    /// Renders the application.
    ///
    /// This should only be called once.
    pub fn run(plugins: Vec<Plugin>) -> Result<()> {
        let (ui_events, plugin_events) = plugins::comm::create_channel(plugins);
        let (ui_events, plugin_events) = (
            Box::leak(Box::new(ui_events)),
            Box::leak(Box::new(plugin_events)),
        );

        let native_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_max_inner_size([WINDOW_WIDTH, MAX_WINDOW_HEIGHT])
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
                Self::load_system_font(ctx);

                Ok(Box::new(Self::new(cc, ui_events, plugin_events)))
            }),
        ) {
            bail!("{e}");
        };

        Ok(())
    }

    fn load_system_font(ctx: &Context) {
        // https://github.com/emilk/egui/discussions/1344#discussioncomment-6432960
        use font_kit::{
            family_name::FamilyName, handle::Handle, properties::Properties, source::SystemSource,
        };

        let mut fonts = FontDefinitions::default();
        let handle = SystemSource::new()
            .select_best_match(&[FamilyName::SansSerif], &Properties::default())
            .expect("could not load system fonts");
        let buf: Vec<u8> = match handle {
            Handle::Path { path, .. } => {
                std::fs::read(path).expect("could not read system font path")
            }
            Handle::Memory { bytes, .. } => bytes.to_vec(),
        };

        const FONT: &str = "System Sans Serif";
        fonts
            .font_data
            .insert(FONT.to_owned(), FontData::from_owned(buf));

        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .map(|vec| vec.insert(0, FONT.to_owned()));

        ctx.set_fonts(fonts);
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

    fn apply_plugin_event(&mut self, ev: PluginEvent, ctx: &Context) {
        match ev {
            PluginEvent::SetList(vec) => {
                self.results = vec;
                self.selection = 0;
            }
            PluginEvent::Activate(evs) => {
                evs.into_iter().for_each(|a| self.apply_activation(a, ctx))
            }
        }
    }

    fn apply_activation(&mut self, activation: PluginActivationAction, ctx: &Context) {
        match activation {
            PluginActivationAction::Close => ctx.send_viewport_cmd(ViewportCommand::Close),
            PluginActivationAction::RunCommandString(cmd) => {
                if let Err(e) = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(&cmd)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                {
                    eprintln!("error running command {cmd:?}: {e}")
                }
            }
            PluginActivationAction::RunCommand((cmd, args)) => {
                if let Err(e) = std::process::Command::new(&cmd)
                    .args(&args)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                {
                    eprintln!("error running command {cmd} {args:?}: {e}")
                }
            }
            PluginActivationAction::Copy(string) => {
                ctx.copy_text(string);
            }
        }
    }

    fn main_ui(&mut self, ui: &mut Ui) {
        // handle special keys first, do not pass them through
        let old_selection = self.selection;
        if consume_input(ui, Key::ArrowDown) {
            self.selection += 1;
        } else if consume_input(ui, Key::ArrowUp) {
            self.selection -= 1;
        } else if consume_input(ui, Key::Enter) {
            if let Some(item) = self.results.get(self.selection).cloned() {
                self.ui_events
                    .send(UiEvent::Activate { item })
                    .expect("ui event receiver must not be closed");
            }
        }
        let selection_changed = old_selection != self.selection;

        // INPUT LINE //

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
                .send(UiEvent::InputChanged {
                    query: self.query.clone(),
                })
                .unwrap();
        }

        // don't add the list if there are no results
        if self.results.is_empty() {
            return;
        }

        // RESULTS LIST //

        ui.add_space(PADDING);
        ui.allocate_ui(
            Vec2::new(ui.available_width(), MAX_WINDOW_HEIGHT - ui.cursor().top()),
            |ui| {
                // TODO: focus flickering if a value is clicked
                ScrollArea::vertical()
                    .max_height(MAX_WINDOW_HEIGHT)
                    .show(ui, |ui| {
                        for (i, item) in self.results.iter().enumerate() {
                            let response = ui.add(Row::new(&mut self.selection, i, item));
                            // make sure the selected item is in view
                            // don't scroll to the item if the user just scrolled away though
                            if self.selection == i && (response.changed() || selection_changed) {
                                response.scroll_to_me(None);
                            }
                        }
                    });
            },
        );
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(ev) = self.plugin_events.try_recv() {
            self.apply_plugin_event(ev, ctx);
        }

        CentralPanel::default()
            .frame(egui::Frame::dark_canvas(&ctx.style()).inner_margin(PADDING))
            .show(ctx, |ui| {
                self.main_ui(ui);

                let existing_height = ctx.input(|i| i.screen_rect.height());
                let new_height = ui.cursor().top() + PADDING;

                if (existing_height - new_height).abs() >= 1.0 {
                    ctx.send_viewport_cmd(ViewportCommand::InnerSize(Vec2::new(
                        WINDOW_WIDTH,
                        new_height,
                    )));
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

struct Row<'sel, 'item, Value> {
    current_value: &'sel mut Value,
    selected_value: Value,
    item: &'item ListItem,
}

impl<'sel, 'item, Value: PartialEq> Row<'sel, 'item, Value> {
    pub fn new(
        current_value: &'sel mut Value,
        selected_value: Value,
        item: &'item ListItem,
    ) -> Self {
        Self {
            current_value,
            selected_value,
            item,
        }
    }
}

impl<Value: PartialEq> Widget for Row<'_, '_, Value> {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let fill = if *self.current_value == self.selected_value {
            ui.visuals().selection.bg_fill
        } else {
            ui.visuals().panel_fill
        };

        let mut contents = LayoutJob::default();
        contents.append(
            &self.item.title,
            0.0,
            TextFormat {
                font_id: ui
                    .style()
                    .text_styles
                    .get(&egui::TextStyle::Body)
                    .unwrap()
                    .clone(),
                ..Default::default()
            },
        );

        if !self.item.description.is_empty() {
            contents.append("\n", 0.0, TextFormat::default());
            contents.append(
                &self.item.description,
                0.0,
                TextFormat {
                    font_id: ui
                        .style()
                        .text_styles
                        .get(&egui::TextStyle::Small)
                        .unwrap()
                        .clone(),
                    color: Color32::GRAY,
                    ..Default::default()
                },
            );
        }

        let mut button = Button::new(contents)
            .frame(false)
            .min_size(Vec2::new(ui.available_width(), 0.0))
            .fill(fill)
            .ui(ui);

        if button.clicked() {
            *self.current_value = self.selected_value;
            button.mark_changed();
            dbg!(button.changed());
        }

        button
    }
}
