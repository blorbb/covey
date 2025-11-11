use std::{collections::BTreeMap, sync::Arc};

use covey::host;
use eframe::CreationContext;
use egui::{
    Color32, CornerRadius, FontFamily, FontId, Key, Margin, RichText, ScrollArea, Stroke, TextEdit,
    TextStyle, Ui, Vec2, Vec2b, style::ScrollAnimation, text::CCursor,
};

use crate::row::ListRow;

pub mod cli;
mod conv;
mod hotkeys;
mod row;
mod style;

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
    style: UserStyle,
    pub settings: GuiSettings,
}

pub struct UserStyle {
    // window
    window_width: f32,
    max_window_height: f32,
    window_margin: f32,
    window_rounding: f32,
    // main colors
    bg_color: Color32,
    text_color: Color32,
    weak_text_color: Color32,
    // fonts
    font_size: f32,
    description_font_size: f32,
    // input
    input_height: f32,
    cursor_selection_bg: Color32,
    input_list_gap: f32,
    // list
    list_item_gap: f32,
    list_selected_color: Color32,
    list_hovered_color: Color32,
    list_rounding: f32,
    list_padding: Vec2,
}

fn alpha(amount: f32) -> u8 {
    ((u8::MAX as f32) * amount) as u8
}

impl Default for UserStyle {
    fn default() -> Self {
        Self {
            window_width: 600.0,
            max_window_height: 500.0,
            window_margin: 12.0,
            input_height: 32.0,
            input_list_gap: 12.0,
            window_rounding: 8.0,
            list_item_gap: 8.0,
            bg_color: Color32::from_rgb(25, 17, 19),
            text_color: Color32::from_white_alpha(alpha(0.85)),
            weak_text_color: Color32::from_white_alpha(alpha(0.6)),
            font_size: 14.0,
            description_font_size: 12.0,
            cursor_selection_bg: Color32::from_rgb(113, 51, 68),
            list_hovered_color: Color32::from_white_alpha(alpha(0.1)),
            list_selected_color: Color32::from_white_alpha(alpha(0.2)),
            list_rounding: 4.0,
            list_padding: Vec2::new(4.0, 2.0),
        }
    }
}

impl UserStyle {
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
        style: UserStyle,
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

        cc.egui_ctx.set_style(style::style_reset());
        cc.egui_ctx.all_styles_mut(|style| {
            let ss = &self.style;

            // window
            style.visuals.window_fill = ss.bg_color;
            style.visuals.window_corner_radius = CornerRadius::same(ss.window_rounding as u8);

            // text colors
            style.visuals.override_text_color = Some(ss.text_color);
            style.visuals.weak_text_color = Some(ss.weak_text_color);

            style.spacing.window_margin = Margin::same(ss.window_margin as i8);
            style.visuals.selection.bg_fill = ss.cursor_selection_bg;

            style.text_styles = BTreeMap::from_iter([
                (
                    TextStyle::Heading,
                    FontId::new(ss.font_size * 2.0, FontFamily::Proportional),
                ),
                (
                    TextStyle::Body,
                    FontId::new(ss.font_size, FontFamily::Proportional),
                ),
                (
                    TextStyle::Monospace,
                    FontId::new(ss.font_size, FontFamily::Monospace),
                ),
                (
                    TextStyle::Button,
                    FontId::new(ss.font_size, FontFamily::Proportional),
                ),
                (
                    TextStyle::Small,
                    FontId::new(ss.description_font_size, FontFamily::Proportional),
                ),
            ]);
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
                        let v = &mut ui.style_mut().visuals;
                        v.selection.bg_fill = self.style.list_selected_color;
                        v.widgets.active.weak_bg_fill = self.style.list_selected_color;
                        v.widgets.hovered.weak_bg_fill = self.style.list_hovered_color;
                        v.widgets
                            .set_corner_radius(CornerRadius::same(self.style.list_rounding as u8));
                        ui.style_mut().spacing.button_padding = self.style.list_padding;

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

                            let is_last = i == list.items.len() - 1;
                            if !is_last {
                                ui.add_space(self.style.list_item_gap);
                            }
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
            .frame(egui::Frame::window(&ctx.style()))
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
                            // Color is not being set correctly for some reason
                            .hint_text(RichText::new("Search...").color(self.style.weak_text_color))
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

#[expect(dead_code)]
trait WidgetsExt {
    fn set_bg(&mut self, bg: Color32);
    fn set_border(&mut self, border: Stroke);
    fn set_corner_radius(&mut self, rad: CornerRadius);
    fn set_expansion(&mut self, expansion: f32);
    fn set_fg(&mut self, fg: Stroke);
}

impl WidgetsExt for egui::style::Widgets {
    fn set_bg(&mut self, bg: Color32) {
        self.active.bg_fill = bg;
        self.hovered.bg_fill = bg;
        self.inactive.bg_fill = bg;
        self.noninteractive.bg_fill = bg;
        self.open.bg_fill = bg;
        self.active.weak_bg_fill = bg;
        self.hovered.weak_bg_fill = bg;
        self.inactive.weak_bg_fill = bg;
        self.noninteractive.weak_bg_fill = bg;
        self.open.weak_bg_fill = bg;
    }

    fn set_border(&mut self, border: Stroke) {
        self.active.bg_stroke = border;
        self.hovered.bg_stroke = border;
        self.inactive.bg_stroke = border;
        self.noninteractive.bg_stroke = border;
        self.open.bg_stroke = border;
    }

    fn set_corner_radius(&mut self, rad: CornerRadius) {
        self.active.corner_radius = rad;
        self.hovered.corner_radius = rad;
        self.inactive.corner_radius = rad;
        self.noninteractive.corner_radius = rad;
        self.open.corner_radius = rad;
    }

    fn set_expansion(&mut self, expansion: f32) {
        self.active.expansion = expansion;
        self.hovered.expansion = expansion;
        self.inactive.expansion = expansion;
        self.noninteractive.expansion = expansion;
        self.open.expansion = expansion;
    }

    fn set_fg(&mut self, fg: Stroke) {
        self.active.fg_stroke = fg;
        self.hovered.fg_stroke = fg;
        self.inactive.fg_stroke = fg;
        self.noninteractive.fg_stroke = fg;
        self.open.fg_stroke = fg;
    }
}
