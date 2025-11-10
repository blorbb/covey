use std::{collections::BTreeMap, sync::Arc};

use covey::host;
use eframe::CreationContext;
use egui::{
    Color32, FontFamily, FontId, Key, ScrollArea, Stroke, TextEdit, TextStyle, Ui, Vec2, Vec2b,
    style::ScrollAnimation, text::CCursor,
};

use crate::row::ListRow;

pub mod cli;
mod conv;
mod hotkeys;
mod row;

pub struct App {
    cli: cli::Receiver,
    tx: host::RequestSender,
    rx: host::ResponseReceiver,
    input: String,
    list: Option<covey::List>,
    list_selection: usize,
    /// Whether the last opening of the app has been focused.
    ///
    /// Used to avoid closing the app early if focus isn't gained for a bit.
    has_focused: bool,
    style: Style,
    pub settings: GuiSettings,
}

pub struct Style {
    window_width: f32,
    max_window_height: f32,
    window_margin: f32,
    input_height: f32,
    input_list_gap: f32,
    window_rounding: f32,
    bg_color: Color32,
    list_item_gap: f32,
    font_size: f32,
    description_font_size: f32,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            window_width: 600.0,
            max_window_height: 500.0,
            window_margin: 12.0,
            input_height: 32.0,
            input_list_gap: 12.0,
            window_rounding: 8.0,
            list_item_gap: 4.0,
            bg_color: Color32::from_rgb(25, 17, 19),
            font_size: 14.0,
            description_font_size: 12.0,
        }
    }
}

impl Style {
    pub fn max_list_height(&self) -> f32 {
        self.max_window_height - 2.0 * self.window_margin - self.input_height - self.input_list_gap
    }

    pub fn inner_width(&self) -> f32 {
        self.window_width - 2.0 * self.window_margin
    }
}

pub struct GuiSettings {
    pub close_on_blur: bool,
}

impl App {
    pub fn new(
        cli_rx: &cli::Receiver,
        style: Style,
        settings: GuiSettings,
    ) -> anyhow::Result<Self> {
        let (mut tx, rx) = covey::host::channel()?;
        // immediately send an empty query
        tokio::spawn(tx.send_query(String::new()));
        Ok(Self {
            cli: cli_rx.clone(),
            tx,
            rx,
            input: String::new(),
            list: None,
            list_selection: 0,
            has_focused: false,
            style,
            settings,
        })
    }

    /// Open the app once, returning when it closes.
    pub fn open(&mut self) -> eframe::Result {
        let options = eframe::NativeOptions {
            run_and_return: true,
            viewport: egui::ViewportBuilder::default()
                .with_resizable(false)
                .with_inner_size([self.style.window_width, self.style.max_window_height])
                .with_active(true)
                .with_transparent(true)
                .with_always_on_top()
                .with_decorations(false),
            ..Default::default()
        };

        let result = eframe::run_native(
            "covey",
            options.clone(),
            Box::new(|cc| {
                self.style_ctx(cc);
                Ok(Box::new(&mut *self))
            }),
        );
        self.has_focused = false;
        result
    }

    fn style_ctx(&self, cc: &CreationContext) {
        // cc.egui_ctx.style_mut_of(egui::Theme::Dark, |style| {});

        cc.egui_ctx.all_styles_mut(|style| {
            style.visuals.panel_fill = self.style.bg_color;
            style.visuals.text_edit_bg_color = Some(self.style.bg_color);

            use FontFamily as FF;
            use TextStyle as TS;
            let ss = &self.style;
            let text_styles = BTreeMap::from_iter([
                (
                    TS::Heading,
                    FontId::new(ss.font_size * 2.0, FF::Proportional),
                ),
                (TS::Body, FontId::new(ss.font_size, FF::Proportional)),
                (TS::Monospace, FontId::new(ss.font_size, FF::Monospace)),
                (TS::Button, FontId::new(ss.font_size, FF::Proportional)),
                (
                    TS::Small,
                    FontId::new(ss.description_font_size, FF::Proportional),
                ),
            ]);

            style.text_styles = text_styles;
        });
    }

    fn show_list(&mut self, ui: &mut Ui, list_selection_changed: bool) {
        let Some(list) = &mut self.list else { return };

        ui.allocate_ui(
            Vec2::new(self.style.inner_width(), self.style.max_list_height()),
            |ui| {
                ScrollArea::vertical()
                    // take up full width but shrink height
                    .auto_shrink(Vec2b::new(false, true))
                    .max_height(self.style.max_list_height())
                    .show(ui, |ui| {
                        let visuals = &mut ui.style_mut().visuals;
                        visuals.selection.bg_fill = Color32::from_white_alpha(u8::MAX / 5);
                        let widget_style = &mut visuals.widgets;
                        widget_style.active.weak_bg_fill = Color32::from_white_alpha(u8::MAX / 5);
                        widget_style.active.bg_stroke = Stroke::NONE;
                        widget_style.hovered.weak_bg_fill = Color32::from_white_alpha(u8::MAX / 10);
                        widget_style.hovered.bg_stroke = Stroke::NONE;
                        widget_style.inactive.weak_bg_fill = Color32::TRANSPARENT;
                        widget_style.inactive.bg_stroke = Stroke::NONE;

                        for (i, item) in list.items.iter().enumerate() {
                            let response = ui.add(ListRow::new(&mut self.list_selection, i, item));

                            // Can't use response.changed() as that
                            // doesn't detect changes to self.selection
                            if self.list_selection == i && list_selection_changed {
                                response.scroll_to_me_animation(
                                    None, // Don't scroll if already visible.
                                    ScrollAnimation::duration(0.2),
                                );
                            }

                            ui.add_space(self.style.list_item_gap);
                        }
                    });
            },
        );
    }
}

impl eframe::App for &mut App {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0; 4]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Don't use CentralPanel as it fills up whatever remaining space there is.
        // Changing the window size has a small delay so theres a big blank empty space
        // for a single frame.
        // Top panel only takes up as much space as the UI needs, so it always has
        // the right size.
        egui::TopBottomPanel::top("main-panel")
            .frame(
                egui::Frame::central_panel(&ctx.style())
                    .inner_margin(self.style.window_margin)
                    .corner_radius(self.style.window_rounding),
            )
            .show(ctx, |ui| {
                if self.settings.close_on_blur {
                    let window_focused = ui.input(|i| i.focused);
                    if window_focused {
                        self.has_focused = true;
                    } else if !window_focused && self.has_focused {
                        tracing::info!("window unfocused");
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        return;
                    }
                }

                // CLI window actions //
                match self.cli.try_recv() {
                    Some(cli::Message::Exit) => {
                        tracing::info!("received exit message");
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        return;
                    }
                    // Trying to open while already open -> do nothing
                    Some(cli::Message::Open) => {}
                    Some(cli::Message::OpenAndStay) => {
                        self.settings.close_on_blur = false;
                    }
                    None => {}
                }

                // covey responses //
                let mut new_selection = None;
                let mut list_selection_changed = false;
                match self.rx.try_recv_action() {
                    None => {}
                    Some(covey::Action::Close) => {
                        tracing::info!("received close request");
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        return;
                    }
                    Some(covey::Action::Copy(str)) => {
                        ui.ctx().send_cmd(egui::OutputCommand::CopyText(str));
                    }
                    Some(covey::Action::DisplayError(title, desc)) => {
                        todo!("error: {title} {desc}")
                    }
                    Some(covey::Action::SetInput(covey::Input {
                        contents,
                        selection,
                    })) => {
                        // Another query to update the plugin on what it changed.
                        // This change isn't detected by text_edit.response.changed()
                        if contents != self.input {
                            tokio::spawn(self.tx.send_query(contents.clone()));
                        }

                        self.input = contents;
                        new_selection = Some((selection.0 as usize, selection.1 as usize));
                    }
                    Some(covey::Action::SetList(list)) => {
                        tracing::debug!("received list with {} items", list.len());
                        self.list = Some(list);
                        self.list_selection = 0;
                    }
                }

                // global keyboard shortcuts //
                if hotkeys::key_pressed_consume(ui, Key::Escape) {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
                if let Some(list) = &self.list {
                    if hotkeys::key_pressed_consume(ui, Key::ArrowDown) {
                        self.list_selection =
                            bounded_wrapping_add(self.list_selection, 1, list.len());
                        list_selection_changed = true;
                    } else if hotkeys::key_pressed_consume(ui, Key::ArrowUp) {
                        self.list_selection =
                            bounded_wrapping_sub(self.list_selection, 1, list.len());
                        list_selection_changed = true;
                    } else if hotkeys::hotkey_pressed_consume(
                        ui,
                        self.tx.config().app.reload_hotkey.clone(),
                    ) {
                        self.tx.reload_plugin(list.plugin.id());
                    }
                }

                // handle activations
                if let Some(list) = &self.list
                    && let Some(item) = list.items.get(self.list_selection)
                    && let Some(hotkey) = hotkeys::hotkey_pressed_now(ui)
                    && let Some(future) = self.tx.activate_by_hotkey(item.clone(), hotkey.clone())
                {
                    hotkeys::hotkey_pressed_consume(ui, hotkey);
                    tokio::spawn(future);
                }

                // the actual UI //

                // text edit

                let row_height =
                    ui.fonts_mut(|f| f.row_height(&egui::TextStyle::Body.resolve(ui.style())));

                let mut text_edit = scope_style(
                    ui,
                    |style| style.visuals.selection.stroke = Stroke::NONE,
                    |ui| {
                        TextEdit::singleline(&mut self.input)
                            .hint_text("Search...")
                            .margin((self.style.input_height - row_height) / 2.0)
                            .desired_width(f32::INFINITY)
                            .return_key(None)
                            .show(ui)
                    },
                );

                if let Some((min, max)) = new_selection {
                    text_edit
                        .state
                        .cursor
                        .set_char_range(Some(egui::text::CCursorRange::two(
                            CCursor::new(min),
                            CCursor::new(max),
                        )));
                    text_edit.state.store(ui.ctx(), text_edit.response.id);
                }

                if text_edit.response.changed() {
                    tokio::spawn(self.tx.send_query(self.input.clone()));
                }

                // can't request focus if the app is unfocused
                if !text_edit.response.has_focus() && ui.input(|i| i.focused) {
                    text_edit.response.request_focus();
                    // the text edit focus ring will flash for one frame without this
                    ui.ctx().request_discard("lost text edit focus");
                }

                // results list
                if let Some(_list) = &mut self.list {
                    ui.add_space(self.style.input_list_gap);

                    self.show_list(ui, list_selection_changed);
                }

                let existing_height = ui.ctx().content_rect().height();
                let new_height = ui.cursor().top() + self.style.window_margin;
                if existing_height != new_height {
                    ui.ctx()
                        .send_viewport_cmd(egui::ViewportCommand::InnerSize(Vec2::new(
                            self.style.window_width,
                            ui.cursor().top() + self.style.window_margin,
                        )));
                }
            });
    }
}

fn bounded_wrapping_add(x: usize, amount: usize, max_excl: usize) -> usize {
    (x + amount) % max_excl
}

fn bounded_wrapping_sub(x: usize, amount: usize, max_excl: usize) -> usize {
    (x + max_excl - (amount % max_excl)) % max_excl
}

fn scope_style<R>(
    ui: &mut Ui,
    mut mutate_style: impl FnMut(&mut egui::Style),
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    let old_style = Arc::clone(ui.style());
    mutate_style(ui.style_mut());
    let result = add_contents(ui);
    ui.set_style(old_style);
    result
}
