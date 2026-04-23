use std::{
    collections::{BTreeMap, HashSet},
    sync::{Arc, LazyLock},
};

use az::SaturatingAs;
use covey::{
    ActivationTarget,
    covey_schema::{hotkey::Hotkey, style::UserStyle},
};
use eframe::CreationContext;
use egui::{
    Align, Color32, CornerRadius, FontFamily, FontId, Key, Layout, Margin, RichText, ScrollArea,
    Sense, Stroke, TextEdit, TextStyle, Ui, Vec2, Vec2b, style::ScrollAnimation, text::CCursor,
    text_edit::TextEditOutput,
};

use crate::{
    row::ListCell,
    widgets::{Container, ImageIcon},
};

pub mod cli;
mod conv;
mod hotkeys;
mod row;
mod style;
pub mod widgets;

static ICON_TEXT_STYLE: LazyLock<TextStyle> = LazyLock::new(|| TextStyle::Name(Arc::from("icon")));
static FONTS: LazyLock<egui::FontDefinitions> = LazyLock::new(style::load_system_fonts);

fn icon_size(ctx: &egui::Context) -> f32 {
    let icon_font = ICON_TEXT_STYLE.resolve(&ctx.global_style());
    ctx.fonts_mut(|f| f.row_height(&icon_font))
}

pub struct App {
    pub host: covey::Host,
    pub cli: cli::Receiver,
    pub plugin_actions: covey::ActionReceiver,
    input: String,
    list: Option<covey::List>,
    list_selection: usize,
    /// Whether the last opening of the app has been focused.
    ///
    /// Used to avoid closing the app early if focus isn't gained for a bit.
    /// Will be false on the first frame of the app opening.
    app_has_been_focused: bool,
    pub gui_settings: GuiSettings,
    /// Whether the gui should be closed.
    ///
    /// After sending a close command, the app continues for a few more updates.
    /// We want the plugin actions received after being closed to be handled by
    /// the background handler (in main.rs) instead of a regular app update.
    ///
    /// Primarily, there are issues with setting the clipboard while the app is
    /// supposed to be closed.
    is_closed: bool,
}

pub struct GuiSettings {
    pub close_on_blur: bool,
}

impl App {
    fn style(&self) -> &UserStyle {
        &self.host.config().style
    }

    pub fn new(cli_rx: cli::Receiver, gui_settings: GuiSettings) -> anyhow::Result<Self> {
        let (mut host, actions) = covey::channel()?;
        // immediately send an empty query
        host.send_query(String::new());
        Ok(Self {
            cli: cli_rx,
            host,
            plugin_actions: actions,
            input: String::new(),
            list: None,
            list_selection: 0,
            app_has_been_focused: false,
            gui_settings,
            is_closed: true,
        })
    }

    /// Open the app once, returning when it closes.
    pub fn open(&mut self) -> eframe::Result {
        let options = eframe::NativeOptions {
            run_and_return: true,
            viewport: egui::ViewportBuilder::default()
                .with_resizable(false)
                // this needs to be the max size that the window could be.
                // otherwise requests to make it larger will be capped at this size.
                // *1.2 to account for minor errors in sizing calculations.
                .with_inner_size([
                    self.style().window_width(),
                    self.style().max_window_height() * 1.2,
                ])
                .with_active(true)
                .with_transparent(true)
                .with_always_on_top()
                .with_decorations(false),
            // TODO: running the closure in eframe::run_native takes ~200ms when using wgpu. Starts
            // up much faster when using glow (~15ms). wgpu seems to be recommended though, so try
            // switch back in the future to see if this gets fixed.
            renderer: eframe::Renderer::Glow,
            ..Default::default()
        };

        self.is_closed = false;
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
        self.is_closed = true;
        result
    }

    fn set_ctx_style(&self, cc: &CreationContext) {
        cc.egui_ctx.set_global_style(style::style_reset());
        cc.egui_ctx.set_fonts(FONTS.clone());
        cc.egui_ctx.all_styles_mut(|style| {
            let ss = self.style();

            // window
            style.visuals.window_fill = ss.bg_color().as_egui();
            style.visuals.window_corner_radius =
                CornerRadius::same(ss.window_rounding().saturating_as());

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

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        // Don't use CentralPanel as it fills up whatever remaining space there is.
        // Changing the window size has a small delay so theres a big blank empty space
        // for a single frame.
        // Top panel only takes up as much space as the UI needs, so it always has
        // the right size.
        egui::Panel::top("main-panel")
            .frame(egui::Frame::window(ui.style()))
            .show_inside(ui, |ui| self.show(ui));
    }
}

/// Changes made within the current frame that the ui elements may need to be
/// aware of.
///
/// These do not persist across multiple frames.
#[derive(Debug)]
pub struct RenderingState {
    pub new_cursor_selection: Option<(usize, usize)>,
    pub list_selection_changed: bool,
    pub window_is_focused: bool,
}

/// Rendering
impl App {
    fn show(&mut self, ui: &mut Ui) {
        if self.is_closed {
            return;
        }

        let mut rendering_state = RenderingState {
            new_cursor_selection: None,
            list_selection_changed: false,
            window_is_focused: ui.input(|i| i.focused),
        };

        while let Some(action) = self.cli.try_recv() {
            match self.handle_cli_action(Some(ui), action) {
                AppControlFlow::Continue | AppControlFlow::OpenGui => {}
                AppControlFlow::CloseGui | AppControlFlow::ExitProcess => return,
            }
        }
        while let Some(action) = self.plugin_actions.try_recv() {
            match self.handle_plugin_action(Some(ui), action, &mut rendering_state) {
                AppControlFlow::Continue | AppControlFlow::OpenGui => {}
                AppControlFlow::CloseGui | AppControlFlow::ExitProcess => return,
            }
        }
        self.handle_keyboard_input(ui, &mut rendering_state);

        // The UI
        self.show_input(ui, &rendering_state);
        ui.add_space(self.style().main_component_gap());
        ui.style_mut().interaction.selectable_labels = false;
        self.show_list(ui, &rendering_state);
        ui.add_space(self.style().main_component_gap());
        self.show_buttom_bar(ui);

        // set window size //
        let existing_height = ui.content_rect().height();
        let new_height = ui.cursor().top() + self.style().window_margin().block;
        if (existing_height - new_height).abs() < 1. {
            ui.send_viewport_cmd(egui::ViewportCommand::InnerSize(Vec2::new(
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
            ui.send_viewport_cmd(egui::ViewportCommand::Close);
            self.is_closed = true;
            return;
        }
    }

    #[tracing::instrument(skip(self, ui))]
    pub fn handle_cli_action(
        &mut self,
        ui: Option<&mut Ui>,
        action: cli::Action,
    ) -> AppControlFlow {
        tracing::trace!("handling cli action");
        match action {
            cli::Action::Exit => {
                tracing::info!("received exit message");
                if let Some(ui) = ui {
                    ui.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                self.is_closed = true;
                AppControlFlow::ExitProcess
            }
            // Trying to open while already open -> do nothing
            cli::Action::OpenAndStay => {
                self.gui_settings.close_on_blur = false;
                AppControlFlow::OpenGui
            }
            cli::Action::Open => AppControlFlow::OpenGui,
        }
    }

    /// If `ui` is [`None`], some actions won't complete or backups will be
    /// used.
    #[tracing::instrument(skip(self, ui, rendering_state))]
    pub fn handle_plugin_action(
        &mut self,
        ui: Option<&mut Ui>,
        action: covey::Action,
        rendering_state: &mut RenderingState,
    ) -> AppControlFlow {
        tracing::trace!("handling plugin action");
        match action {
            covey::Action::Close => {
                tracing::info!("received close request");
                if let Some(ui) = ui {
                    ui.send_viewport_cmd(egui::ViewportCommand::Close);
                };
                self.is_closed = true;
                AppControlFlow::CloseGui
            }
            covey::Action::Copy(str) => {
                if let Some(ui) = ui {
                    ui.copy_text(str);
                } else {
                    _ = arboard::Clipboard::new()
                        .and_then(|mut c| c.set_text(str))
                        .inspect_err(|e| tracing::error!("failed to set clipboard text: {e:#}"));
                }
                AppControlFlow::Continue
            }
            covey::Action::DisplayError(title, desc) => {
                tokio::spawn(async move {
                    _ = notify_rust::Notification::new()
                        .summary(&title)
                        .body(&desc)
                        .show_async()
                        .await
                        .inspect_err(|e| {
                            tracing::error!("failed to display error notification: {e:#}");
                            tracing::error!("notification was:\n{title}\n{desc}");
                        });
                });
                AppControlFlow::Continue
            }
            covey::Action::SetInput(covey::Input {
                contents,
                selection: (min, max),
            }) => {
                // Another query to update the plugin on what it changed.
                // This change isn't detected by text_edit.response.changed()
                if contents != self.input {
                    self.host.send_query(contents.clone());
                }
                self.input = contents;
                rendering_state.new_cursor_selection = Some((min, max));
                AppControlFlow::Continue
            }
            covey::Action::SetList(list) => {
                tracing::debug!("received list with {} items", list.total_len());
                self.list = Some(list);
                self.list_selection = 0;
                rendering_state.list_selection_changed = true;
                AppControlFlow::Continue
            }
        }
    }

    fn handle_keyboard_input(&mut self, ui: &mut Ui, rendering_state: &mut RenderingState) {
        // global hotkeys

        if hotkeys::key_pressed_consume(ui, Key::Escape) {
            ui.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        if let Some(list) = &self.list {
            if hotkeys::key_pressed_consume(ui, Key::ArrowDown) {
                self.list_selection =
                    bounded_wrapping_add(self.list_selection, 1, list.total_len());
                rendering_state.list_selection_changed = true;
            } else if hotkeys::key_pressed_consume(ui, Key::ArrowUp) {
                self.list_selection =
                    bounded_wrapping_sub(self.list_selection, 1, list.total_len());
                rendering_state.list_selection_changed = true;
            } else if hotkeys::hotkey_pressed_consume(ui, self.host.config().app.reload_hotkey) {
                let plugin_to_reload = list.plugin().id().clone();
                // avoid activating now stale items
                self.list = None;
                self.host.reload_plugin(&plugin_to_reload);
                self.host.send_query(self.input.clone());
            }
        }

        // activations

        if let Some(list) = &self.list
            && let Some(hotkey) = hotkeys::hotkey_pressed_now(ui)
        {
            // list commands are lower priority than list item commands.
            let activated_command = list
                .get_item(self.list_selection)
                .and_then(|item| {
                    self.host
                        .activate_by_hotkey(item.activation_target(), hotkey)
                })
                .or_else(|| {
                    self.host
                        .activate_by_hotkey(list.activation_target(), hotkey)
                });

            if activated_command.is_some() {
                hotkeys::hotkey_pressed_consume(ui, hotkey);
            }
        }
    }

    fn show_input(&mut self, ui: &mut Ui, rendering_state: &RenderingState) {
        ui.spacing_mut().item_spacing = Vec2::ZERO;
        let mut text_edit = Container::new()
            .exact_height(self.style().input_height())
            .inner_margin(Margin::symmetric(
                self.style().list_item_padding().inline.saturating_as(),
                0,
            ))
            .sense(Sense::empty())
            .show_with_layout(ui, Layout::left_to_right(Align::Center), |ui| {
                ui.spacing_mut().item_spacing = self.style().list_item_padding().as_egui();

                // search or loading icon
                let icon_size = crate::icon_size(ui);
                Container::new()
                    .exact_size(Vec2::splat(icon_size))
                    .sense(Sense::empty())
                    .show(ui, |ui| {
                        if self
                            .list
                            .as_ref()
                            .is_none_or(|l| l.is_response_of_latest_query(&self.host))
                        {
                            if let Ok(icon) = ImageIcon::from_icon_name(
                                &self.host,
                                "search",
                                Vec2::splat(icon_size),
                            ) {
                                ui.centered_and_justified(|ui| ui.add(icon));
                            } else {
                                // leave a gap so the loading icon doesn't move
                                // the input around.
                            }
                        } else {
                            // spinner that encompasses the entire icon rect looks too big
                            let spinner_scale = 0.6;
                            ui.centered_and_justified(|ui| {
                                ui.add(
                                    egui::Spinner::new()
                                        .size(icon_size * spinner_scale)
                                        .color(self.style().text_color().as_egui()),
                                )
                            });
                        }
                    });

                // a stable id is needed or else the changing icons above makes this buggy
                let hint_text_color = self.style().weak_text_color().as_egui();
                TextEdit::singleline(&mut self.input)
                    .id_salt("covey main input")
                    .hint_text(RichText::new("Search...").color(hint_text_color))
                    .margin(Margin::ZERO)
                    .desired_width(f32::INFINITY)
                    .return_key(None)
                    .show(ui)
            })
            .inner;

        // Misc state changes
        if !self.app_has_been_focused {
            text_edit.select_all(ui); // select all on first frame
        }

        if let Some((min, max)) = rendering_state.new_cursor_selection {
            text_edit.set_cursor_selection(ui, min, max);
        }

        if text_edit.response.changed() {
            self.host.send_query(self.input.clone());
        }

        // can't request focus if the app is unfocused
        if !text_edit.response.has_focus() && rendering_state.window_is_focused {
            text_edit.response.request_focus();
            // the text edit focus ring will flash for one frame without this
            ui.request_discard("lost text edit focus");
        }
    }

    fn show_list(&mut self, ui: &mut Ui, rendering_state: &RenderingState) {
        // need to manually unpack for disjoint borrows
        let Some(list) = &mut self.list else { return };
        let s = &self.host.config().style;

        ui.allocate_ui(Vec2::new(s.inner_width(), s.max_list_height()), |ui| {
            ScrollArea::vertical()
                // take up full width but shrink height
                .auto_shrink(Vec2b::new(false, true))
                .max_height(s.max_list_height())
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = Vec2::splat(s.list_item_gap());

                    let mut passed_items = 0;
                    for section in &list.sections {
                        if !section.title.is_empty() {
                            ui.label(&section.title);
                        }

                        for (i, item) in section.items.iter().enumerate() {
                            let i = i + passed_items;
                            let response = ListCell::new(&mut self.list_selection, i, item)
                                .show(&self.host, ui, s);

                            if rendering_state.list_selection_changed && i == self.list_selection {
                                tracing::info!("list selection changed");
                                response.scroll_to_me_animation(
                                    None, // Don't scroll if already visible.
                                    ScrollAnimation::duration(0.2),
                                );
                            }
                        }

                        passed_items += section.items.len();
                    }
                })
        });
    }

    fn show_buttom_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            match &self.list {
                None => {
                    ui.label("No plugin activated");
                }
                Some(list) => {
                    ui.spacing_mut().item_spacing = Vec2::splat(self.style().info_button_gap());

                    // some commands have the same hotkey, but we only activate the FIRST
                    // command with the given hotkey. need to filter out duplicate hotkeys from
                    // subsequent commands.
                    let mut used_hotkeys = HashSet::<Hotkey>::new();

                    if let Some(selected_item) = list.get_item(self.list_selection) {
                        show_command_buttons(
                            &mut self.host,
                            ui,
                            selected_item.activation_target(),
                            &mut used_hotkeys,
                        );
                    };

                    // list commands are lower priority than list item commands.
                    show_command_buttons(
                        &mut self.host,
                        ui,
                        list.activation_target(),
                        &mut used_hotkeys,
                    );

                    // TODO: make clicking this go to a settings window
                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                        ui.add(egui::Button::new(&list.plugin().manifest().name));
                    });
                }
            }
        });

        fn show_command_buttons(
            host: &mut covey::Host,
            ui: &mut Ui,
            activation_target: &ActivationTarget,
            used_hotkeys: &mut HashSet<Hotkey>,
        ) {
            for command in activation_target.available_commands() {
                let s = &host.config().style;

                let button = Container::new()
                    .inner_margin(s.info_button_padding().as_egui())
                    .corner_radius(s.info_button_rounding().into())
                    .fill(s.info_button_bg().as_egui())
                    .hover_fill(s.info_button_hovered_bg().as_egui())
                    .active_fill(s.info_button_active_bg().as_egui())
                    .show_horizontal(ui, |ui| {
                        // space between command and shortcut
                        ui.style_mut().spacing.item_spacing = s.info_button_padding().as_egui();
                        ui.label(&command.title);

                        if let Some(hotkeys) =
                            activation_target.plugin().hotkeys_of_cmd(&command.id)
                            && let Some(new_hotkey) =
                                hotkeys.iter().find(|hotkey| !used_hotkeys.contains(hotkey))
                        {
                            ui.colored_label(
                                s.weak_text_color().as_egui(),
                                new_hotkey.to_short_string(),
                            );
                            used_hotkeys.extend(hotkeys);
                        }
                    });

                if button.response.clicked() {
                    host.activate(activation_target, &command.id);
                }
            }
        }
    }
}

/// Control flow of the application that should be taken after an action has
/// been handled.
#[must_use = "additional control flow must be handled by caller"]
pub enum AppControlFlow {
    /// Let the app continue as normal (open/closed stays as is).
    Continue,
    /// Open the GUI if the window is not already open.
    OpenGui,
    /// Close the GUI but continue waiting for more commands in the background.
    CloseGui,
    /// Close the GUI and exit the entire process.
    ExitProcess,
}

fn bounded_wrapping_add(x: usize, amount: usize, max_excl: usize) -> usize {
    if max_excl == 0 {
        return 0;
    }
    (x + amount) % max_excl
}

fn bounded_wrapping_sub(x: usize, amount: usize, max_excl: usize) -> usize {
    if max_excl == 0 {
        return 0;
    }
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
        self.state.clone().store(ui, self.response.id);
    }

    fn select_all(&mut self, ui: &mut Ui) {
        self.state
            .cursor
            .set_char_range(Some(egui::text::CCursorRange::select_all(&self.galley)));
        self.state.clone().store(ui, self.response.id);
    }
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
        Margin::symmetric(
            self.inline.round().saturating_as(),
            self.block.round().saturating_as(),
        )
    }
}

impl AsEgui<Vec2> for covey::covey_schema::style::Padding {
    fn as_egui(&self) -> Vec2 {
        Vec2::new(self.inline, self.block)
    }
}
