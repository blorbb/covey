use std::{
    collections::BTreeMap,
    ops::ControlFlow,
    sync::{Arc, LazyLock},
};

use covey::covey_schema::style::UserStyle;
use eframe::CreationContext;
use egui::{
    Color32, CornerRadius, FontFamily, FontId, Key, Margin, RichText, ScrollArea, Stroke, TextEdit,
    TextStyle, Ui, Vec2, Vec2b, style::ScrollAnimation, text::CCursor, text_edit::TextEditOutput,
};

use crate::row::ListRow;

pub mod cli;
mod conv;
mod hotkeys;
mod row;
mod style;

static ICON_TEXT_STYLE: LazyLock<TextStyle> = LazyLock::new(|| TextStyle::Name(Arc::from("icon")));
static FONTS: LazyLock<egui::FontDefinitions> = LazyLock::new(style::load_system_fonts);

pub struct App {
    cli: cli::Receiver,
    host: covey::Host,
    actions: covey::ActionReceiver,
    input: String,
    list: Option<covey::List>,
    list_selection: usize,
    /// Whether the last opening of the app has been focused.
    ///
    /// Used to avoid closing the app early if focus isn't gained for a bit.
    /// Will be false on the first frame of the app opening.
    app_has_been_focused: bool,
    pub gui_settings: GuiSettings,
}

pub struct GuiSettings {
    pub close_on_blur: bool,
}

impl App {
    fn style(&self) -> &UserStyle {
        &self.host.config().style
    }

    pub fn new(cli_rx: &cli::Receiver, gui_settings: GuiSettings) -> anyhow::Result<Self> {
        let (mut host, actions) = covey::channel()?;
        // immediately send an empty query
        tokio::spawn(host.send_query(String::new()));
        Ok(Self {
            cli: cli_rx.clone(),
            host,
            actions,
            input: String::new(),
            list: None,
            list_selection: 0,
            app_has_been_focused: false,
            gui_settings,
        })
    }

    /// Open the app once, returning when it closes.
    pub fn open(&mut self) -> eframe::Result {
        let options = eframe::NativeOptions {
            run_and_return: true,
            viewport: egui::ViewportBuilder::default()
                .with_resizable(false)
                .with_inner_size([
                    self.style().window_width(),
                    self.style().max_window_height() * 1.2,
                ])
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
                egui_extras::install_image_loaders(&cc.egui_ctx);
                self.set_ctx_style(cc);
                Ok(Box::new(&mut *self))
            }),
        );
        self.app_has_been_focused = false;
        result
    }

    fn set_ctx_style(&self, cc: &CreationContext) {
        cc.egui_ctx.set_fonts(FONTS.clone());
        cc.egui_ctx.set_style(style::style_reset());
        cc.egui_ctx.all_styles_mut(|style| {
            let ss = self.style();

            // window
            style.visuals.window_fill = ss.bg_color().as_egui();
            style.visuals.window_corner_radius = CornerRadius::same(ss.window_rounding() as u8);

            // text colors
            style.visuals.override_text_color = Some(ss.text_color().as_egui());
            style.visuals.weak_text_color = Some(ss.weak_text_color().as_egui());

            style.spacing.window_margin = ss.window_margin().as_egui();
            style.visuals.selection.bg_fill = ss.cursor_selection_bg().as_egui();

            style.text_styles = BTreeMap::from_iter([
                (
                    TextStyle::Heading,
                    FontId::new(ss.font_size() * 2.0, FontFamily::Proportional),
                ),
                (
                    TextStyle::Body,
                    FontId::new(ss.font_size(), FontFamily::Proportional),
                ),
                (
                    TextStyle::Monospace,
                    FontId::new(ss.font_size(), FontFamily::Monospace),
                ),
                (
                    TextStyle::Button,
                    FontId::new(ss.font_size(), FontFamily::Proportional),
                ),
                (
                    TextStyle::Small,
                    FontId::new(ss.description_font_size(), FontFamily::Proportional),
                ),
                (
                    ICON_TEXT_STYLE.clone(),
                    FontId::new(
                        ss.font_size() + ss.description_font_size(),
                        FontFamily::Proportional,
                    ),
                ),
            ]);
        });
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
                let _: ControlFlow<()> = self.show(ui);
            });
    }
}

/// Changes made within the current frame that the ui elements may need to be
/// aware of.
///
/// These do not persist across multiple frames.
struct RenderingState {
    new_cursor_selection: Option<(usize, usize)>,
    list_selection_changed: bool,
    window_is_focused: bool,
}

/// Rendering
impl App {
    /// The returned [`ControlFlow`] doesn't mean anything, just convenient for
    /// using `?` operator.
    fn show(&mut self, ui: &mut Ui) -> ControlFlow<()> {
        let mut rendering_state = RenderingState {
            new_cursor_selection: None,
            list_selection_changed: false,
            window_is_focused: ui.input(|i| i.focused),
        };

        self.handle_cli_action(ui)?;
        self.handle_plugin_action(ui, &mut rendering_state)?;
        self.handle_keyboard_input(ui, &mut rendering_state);

        // The UI
        self.show_input(ui, &rendering_state);
        self.show_list(ui, &rendering_state);
        self.show_buttom_bar(ui);

        // set window size //
        let existing_height = ui.ctx().content_rect().height();
        let new_height = ui.cursor().top() + self.style().window_margin().block;
        if (existing_height - new_height).abs() < 1. {
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::InnerSize(Vec2::new(
                    self.style().window_width(),
                    new_height,
                )));
        }

        // Close if unfocused
        // must set this at the end to guarantee it is false on the first frame
        self.app_has_been_focused |= rendering_state.window_is_focused;
        // this must also be at the end to avoid a weird flash of blank when closing
        // by losing focus
        if self.gui_settings.close_on_blur
            && self.app_has_been_focused
            && !rendering_state.window_is_focused
        {
            tracing::info!("window unfocused");
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            return ControlFlow::Break(());
        }

        ControlFlow::Continue(())
    }

    fn handle_cli_action(&mut self, ui: &mut Ui) -> ControlFlow<()> {
        match self.cli.try_recv() {
            Some(cli::Message::Exit) => {
                tracing::info!("received exit message");
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                ControlFlow::Break(())
            }
            // Trying to open while already open -> do nothing
            Some(cli::Message::OpenAndStay) => {
                self.gui_settings.close_on_blur = false;
                ControlFlow::Continue(())
            }
            Some(cli::Message::Open) => ControlFlow::Continue(()),
            None => ControlFlow::Continue(()),
        }
    }

    fn handle_plugin_action(
        &mut self,
        ui: &mut Ui,
        rendering_state: &mut RenderingState,
    ) -> ControlFlow<()> {
        match self.actions.try_recv_action(&self.host) {
            None => ControlFlow::Continue(()),
            Some(covey::Action::Close) => {
                tracing::info!("received close request");
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                ControlFlow::Break(())
            }
            Some(covey::Action::Copy(str)) => {
                ui.ctx().send_cmd(egui::OutputCommand::CopyText(str));
                ControlFlow::Continue(())
            }
            Some(covey::Action::DisplayError(title, desc)) => {
                todo!("error: {title} {desc}");
            }
            Some(covey::Action::SetInput(covey::Input {
                contents,
                selection: (min, max),
            })) => {
                // Another query to update the plugin on what it changed.
                // This change isn't detected by text_edit.response.changed()
                if contents != self.input {
                    tokio::spawn(self.host.send_query(contents.clone()));
                }
                self.input = contents;
                rendering_state.new_cursor_selection = Some((min, max));
                ControlFlow::Continue(())
            }
            Some(covey::Action::SetList(list)) => {
                tracing::debug!("received list with {} items", list.len());
                self.list = Some(list);
                self.list_selection = 0;
                rendering_state.list_selection_changed = true;
                ControlFlow::Continue(())
            }
        }
    }

    fn handle_keyboard_input(&mut self, ui: &mut Ui, rendering_state: &mut RenderingState) {
        // global hotkeys

        if hotkeys::key_pressed_consume(ui, Key::Escape) {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }

        if let Some(list) = &self.list {
            if hotkeys::key_pressed_consume(ui, Key::ArrowDown) {
                self.list_selection = bounded_wrapping_add(self.list_selection, 1, list.len());
                rendering_state.list_selection_changed = true;
            } else if hotkeys::key_pressed_consume(ui, Key::ArrowUp) {
                self.list_selection = bounded_wrapping_sub(self.list_selection, 1, list.len());
                rendering_state.list_selection_changed = true;
            } else if hotkeys::hotkey_pressed_consume(
                ui,
                self.host.config().app.reload_hotkey.clone(),
            ) {
                self.host.reload_plugin(list.plugin.id());
                tokio::spawn(self.host.send_query(self.input.clone()));
            }
        }

        // activations

        if let Some(list) = &self.list
            && let Some(item) = list.items.get(self.list_selection)
            && let Some(hotkey) = hotkeys::hotkey_pressed_now(ui)
            && let Some(future) = self.host.activate_by_hotkey(item.clone(), hotkey.clone())
        {
            hotkeys::hotkey_pressed_consume(ui, hotkey);
            tokio::spawn(future);
        }
    }

    fn show_input(&mut self, ui: &mut Ui, rendering_state: &RenderingState) {
        let row_height = ui.fonts_mut(|f| f.row_height(&egui::TextStyle::Body.resolve(ui.style())));

        let mut text_edit = scope_style(
            ui,
            |style| style.visuals.selection.stroke = Stroke::NONE,
            |ui| {
                // need disjoint borrows
                let style = &self.host.config().style;
                TextEdit::singleline(&mut self.input)
                    // Color is not being set correctly for some reason
                    .hint_text(RichText::new("Search...").color(style.weak_text_color().as_egui()))
                    .margin((style.input_height() - row_height) / 2.0)
                    .desired_width(f32::INFINITY)
                    .return_key(None)
                    .show(ui)
            },
        );

        // Misc state changes
        if !self.app_has_been_focused {
            text_edit.select_all(ui); // select all on first frame
        }

        if let Some((min, max)) = rendering_state.new_cursor_selection {
            text_edit.set_cursor_selection(ui, min, max);
        }

        if text_edit.response.changed() {
            tokio::spawn(self.host.send_query(self.input.clone()));
        }

        // can't request focus if the app is unfocused
        if !text_edit.response.has_focus() && rendering_state.window_is_focused {
            text_edit.response.request_focus();
            // the text edit focus ring will flash for one frame without this
            ui.ctx().request_discard("lost text edit focus");
        }
    }

    fn show_list(&mut self, ui: &mut Ui, rendering_state: &RenderingState) {
        // need to manually unpack for disjoint borrows
        let Some(list) = &mut self.list else { return };
        let s = &self.host.config().style;

        ui.add_space(s.main_component_gap());

        ui.allocate_ui(Vec2::new(s.inner_width(), s.max_list_height()), |ui| {
            ScrollArea::vertical()
                // take up full width but shrink height
                .auto_shrink(Vec2b::new(false, true))
                .max_height(s.max_list_height())
                .show(ui, |ui| {
                    let v = &mut ui.style_mut().visuals;
                    v.selection.bg_fill = s.list_selected_color().as_egui();
                    v.widgets.active.weak_bg_fill = s.list_selected_color().as_egui();
                    v.widgets.hovered.weak_bg_fill = s.list_hovered_color().as_egui();
                    v.widgets
                        .set_corner_radius(CornerRadius::same(s.list_rounding().round() as u8));
                    ui.style_mut().spacing.button_padding = s.list_padding().as_egui();
                    ui.style_mut().spacing.icon_spacing = s.list_padding().block;

                    for (i, item) in list.items.iter().enumerate() {
                        let response = ui.add(ListRow::new(&mut self.list_selection, i, item));

                        if rendering_state.list_selection_changed && i == self.list_selection {
                            tracing::info!("list selection changed");
                            response.scroll_to_me_animation(
                                None, // Don't scroll if already visible.
                                ScrollAnimation::duration(0.2),
                            );
                        }

                        let is_last = i == list.items.len() - 1;
                        if !is_last {
                            ui.add_space(s.list_item_gap());
                        }
                    }
                })
        });
    }

    fn show_buttom_bar(&mut self, ui: &mut Ui) {
        ui.add_space(self.style().window_margin().block);
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
        let v = &mut ui.style_mut().visuals;
        let s = self.style();
        v.selection.bg_fill = s.list_selected_color().as_egui();
        v.widgets.active.weak_bg_fill = s.list_selected_color().as_egui();
        v.widgets.hovered.weak_bg_fill = s.list_hovered_color().as_egui();
        v.widgets
            .set_corner_radius(CornerRadius::same(s.list_rounding().round() as u8));
        ui.style_mut().spacing.button_padding = s.list_padding().as_egui();
        ui.style_mut().spacing.icon_spacing = s.list_padding().block;
        ui.style_mut().spacing.item_spacing = Vec2::splat(s.list_item_gap());

        ui.horizontal(|ui| {
            match &self.list {
                None => {
                    ui.label("No plugin activated");
                }
                Some(list) => {
                    if let Some(selected_item) = list.items.get(self.list_selection) {
                        selected_item.available_commands().for_each(|command| {
                            // TODO: handle clicks
                            let button = egui::Button::new(&command.title);
                            let button = if let Some([hotkey, ..]) =
                                selected_item.plugin().hotkeys_of_cmd(&command.id)
                            {
                                button.shortcut_text(hotkey.to_short_string())
                            } else {
                                button
                            };
                            ui.add(button);
                        });
                    };
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add(egui::Button::new(&list.plugin.manifest().name));
                    });
                }
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

trait TextEditExt {
    fn set_cursor_selection(&mut self, ui: &mut Ui, min: usize, max: usize);
    fn select_all(&mut self, ui: &mut Ui);
}

impl TextEditExt for TextEditOutput {
    fn set_cursor_selection(&mut self, ui: &mut Ui, min: usize, max: usize) {
        self.state
            .cursor
            .set_char_range(Some(egui::text::CCursorRange::two(
                CCursor::new(min),
                CCursor::new(max),
            )));
        self.state.clone().store(ui.ctx(), self.response.id);
    }

    fn select_all(&mut self, ui: &mut Ui) {
        self.state
            .cursor
            .set_char_range(Some(egui::text::CCursorRange::select_all(&self.galley)));
        self.state.clone().store(ui.ctx(), self.response.id);
    }
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

trait AsEgui<T> {
    fn as_egui(&self) -> T;
}

impl AsEgui<Color32> for covey::covey_schema::style::Color {
    fn as_egui(&self) -> Color32 {
        let [r, g, b, a] = self.split_rgba();
        Color32::from_rgba_unmultiplied(r, g, b, a)
    }
}

impl AsEgui<Margin> for covey::covey_schema::style::Padding {
    fn as_egui(&self) -> Margin {
        Margin::symmetric(self.inline.round() as i8, self.block.round() as i8)
    }
}

impl AsEgui<Vec2> for covey::covey_schema::style::Padding {
    fn as_egui(&self) -> Vec2 {
        Vec2::new(self.inline, self.block)
    }
}
