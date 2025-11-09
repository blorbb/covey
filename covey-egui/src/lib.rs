use covey::{
    covey_schema::hotkey::{Hotkey, KeyCode},
    host,
};
use egui::{
    Key, KeyboardShortcut, Modifiers, ScrollArea, TextEdit, Ui, Vec2, Vec2b,
    style::ScrollAnimation, text::CCursor,
};

pub mod cli;

pub struct App {
    cli: cli::Receiver,
    tx: host::RequestSender,
    rx: host::ResponseReceiver,
    input: String,
    list: Option<covey::List>,
    list_selection: usize,
    style: Style,
}

pub struct Style {
    window_width: f32,
    max_window_height: f32,
    window_margin: f32,
    input_height: f32,
    input_list_gap: f32,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            window_width: 600.0,
            max_window_height: 500.0,
            window_margin: 12.0,
            input_height: 32.0,
            input_list_gap: 12.0,
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

impl App {
    pub fn new(cli_rx: &cli::Receiver, style: Style) -> anyhow::Result<Self> {
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
            style,
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

        eframe::run_native("covey", options.clone(), Box::new(|_cc| Ok(Box::new(self))))
    }
}

impl eframe::App for &mut App {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0; 4]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::dark_canvas(&ctx.style()).inner_margin(self.style.window_margin))
            .show(ctx, |ui| {
                // CLI window actions //
                match self.cli.try_recv() {
                    Some(cli::Message::Exit) => {
                        tracing::info!("received exit message");
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        // return;
                    }
                    // Trying to open while already open -> do nothing
                    Some(cli::Message::Open) => {}
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
                        // return;
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
                if key_pressed_consume(ui, Key::Escape) {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
                if let Some(list) = &self.list {
                    if key_pressed_consume(ui, Key::ArrowDown) {
                        self.list_selection =
                            bounded_wrapping_add(self.list_selection, 1, list.len());
                        list_selection_changed = true;
                    } else if key_pressed_consume(ui, Key::ArrowUp) {
                        self.list_selection =
                            bounded_wrapping_sub(self.list_selection, 1, list.len());
                        list_selection_changed = true;
                    } else if hotkey_pressed_consume(ui, self.tx.config().app.reload_hotkey.clone())
                    {
                        self.tx.reload_plugin(list.plugin.id());
                    }
                }

                // handle activations
                if let Some(list) = &self.list
                    && let Some(item) = list.items.get(self.list_selection)
                    && let Some(hotkey) = get_hotkey(ui)
                    && let Some(future) = self.tx.activate_by_hotkey(item.clone(), hotkey.clone())
                {
                    hotkey_pressed_consume(ui, hotkey);
                    tokio::spawn(future);
                }

                // the actual UI //

                // text edit

                let row_height =
                    ui.fonts_mut(|f| f.row_height(&egui::TextStyle::Body.resolve(ui.style())));

                let mut text_edit = TextEdit::singleline(&mut self.input)
                    .hint_text("Search...")
                    .margin((self.style.input_height - row_height) / 2.0)
                    .desired_width(f32::INFINITY)
                    .return_key(None)
                    .show(ui);

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

                if !text_edit.response.has_focus() {
                    text_edit.response.request_focus();
                    // the text edit focus ring will flash for one frame without this
                    ui.ctx().request_discard("lost text edit focus");
                }

                // results list
                if let Some(list) = &mut self.list {
                    ui.add_space(self.style.input_list_gap);

                    ui.allocate_ui(
                        Vec2::new(self.style.inner_width(), self.style.max_list_height()),
                        |ui| {
                            ScrollArea::vertical()
                                // take up full width but shrink height
                                .auto_shrink(Vec2b::new(false, true))
                                .max_height(self.style.max_list_height())
                                .show(ui, |ui| {
                                    for (i, item) in list.items.iter().enumerate() {
                                        let response = ui.radio_value(
                                            &mut self.list_selection,
                                            i,
                                            item.title(),
                                        );

                                        // Can't use response.changed() as that
                                        // doesn't detect changes to self.selection
                                        if self.list_selection == i && list_selection_changed {
                                            response.scroll_to_me_animation(
                                                None, // Don't scroll if already visible.
                                                ScrollAnimation::duration(0.2),
                                            );
                                        }
                                    }
                                });
                        },
                    );
                    ui.end_row();
                }

                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::InnerSize(Vec2::new(
                        self.style.window_width,
                        ui.cursor().top() + self.style.window_margin,
                    )));
            });
    }
}

fn key_pressed_consume(ui: &mut Ui, key: Key) -> bool {
    ui.input_mut(|state| state.consume_key(Modifiers::NONE, key))
}

fn hotkey_pressed_consume(ui: &mut Ui, key: Hotkey) -> bool {
    let is_mac = ui.ctx().os().is_mac();
    ui.input_mut(|state| {
        state.consume_shortcut(&KeyboardShortcut::new(
            Modifiers {
                alt: key.alt,
                ctrl: key.ctrl,
                shift: key.shift,
                mac_cmd: is_mac && key.ctrl,
                command: key.ctrl,
            },
            covey_key_code_to_egui_key(key.key),
        ))
    })
}

fn bounded_wrapping_add(x: usize, amount: usize, max_excl: usize) -> usize {
    (x + amount) % max_excl
}

fn bounded_wrapping_sub(x: usize, amount: usize, max_excl: usize) -> usize {
    (x + max_excl - (amount % max_excl)) % max_excl
}

fn get_hotkey(ui: &mut Ui) -> Option<Hotkey> {
    ui.input(|i| {
        let keys_pressed: Vec<_> = i
            .events
            .iter()
            .filter_map(|ev| match ev {
                egui::Event::Key {
                    key,
                    physical_key: _,
                    pressed: true,
                    repeat: false,
                    modifiers: _,
                } => Some(key),
                _ => None,
            })
            .collect();

        if keys_pressed.len() > 1 {
            return None;
        }

        let key_code = egui_key_to_covey_key_code(**keys_pressed.first()?)?;

        let m = i.modifiers;
        Some(Hotkey {
            key: key_code,
            ctrl: m.command,
            alt: m.alt,
            shift: m.shift,
            meta: false,
        })
    })
}

fn egui_key_to_covey_key_code(key: Key) -> Option<KeyCode> {
    match key {
        Key::ArrowDown => None,
        Key::ArrowLeft => None,
        Key::ArrowRight => None,
        Key::ArrowUp => None,
        Key::Escape => None,
        Key::Tab => Some(KeyCode::Tab),
        Key::Backspace => None,
        Key::Enter => Some(KeyCode::Enter),
        Key::Space => None,
        Key::Insert => None,
        Key::Delete => None,
        Key::Home => None,
        Key::End => None,
        Key::PageUp => None,
        Key::PageDown => None,
        Key::Copy => None,
        Key::Cut => None,
        Key::Paste => None,
        Key::Colon => None,
        Key::Comma => Some(KeyCode::Comma),
        Key::Backslash => Some(KeyCode::Backslash),
        Key::Slash => Some(KeyCode::Slash),
        // TODO: eventually support different keyboards that might have these?
        Key::Pipe => None,
        Key::Questionmark => None,
        Key::Exclamationmark => None,
        Key::OpenBracket => Some(KeyCode::LeftBracket),
        Key::CloseBracket => Some(KeyCode::RightBracket),
        Key::OpenCurlyBracket => None,
        Key::CloseCurlyBracket => None,
        Key::Backtick => Some(KeyCode::Backtick),
        Key::Minus => Some(KeyCode::Hyphen),
        Key::Period => Some(KeyCode::Period),
        Key::Plus => None,
        Key::Equals => Some(KeyCode::Equal),
        Key::Semicolon => Some(KeyCode::Semicolon),
        Key::Quote => Some(KeyCode::Apostrophe),
        Key::Num0 => Some(KeyCode::Digit0),
        Key::Num1 => Some(KeyCode::Digit1),
        Key::Num2 => Some(KeyCode::Digit2),
        Key::Num3 => Some(KeyCode::Digit3),
        Key::Num4 => Some(KeyCode::Digit4),
        Key::Num5 => Some(KeyCode::Digit5),
        Key::Num6 => Some(KeyCode::Digit6),
        Key::Num7 => Some(KeyCode::Digit7),
        Key::Num8 => Some(KeyCode::Digit8),
        Key::Num9 => Some(KeyCode::Digit9),
        Key::A => Some(KeyCode::A),
        Key::B => Some(KeyCode::B),
        Key::C => Some(KeyCode::C),
        Key::D => Some(KeyCode::D),
        Key::E => Some(KeyCode::E),
        Key::F => Some(KeyCode::F),
        Key::G => Some(KeyCode::G),
        Key::H => Some(KeyCode::H),
        Key::I => Some(KeyCode::I),
        Key::J => Some(KeyCode::J),
        Key::K => Some(KeyCode::K),
        Key::L => Some(KeyCode::L),
        Key::M => Some(KeyCode::M),
        Key::N => Some(KeyCode::N),
        Key::O => Some(KeyCode::O),
        Key::P => Some(KeyCode::P),
        Key::Q => Some(KeyCode::Q),
        Key::R => Some(KeyCode::R),
        Key::S => Some(KeyCode::S),
        Key::T => Some(KeyCode::T),
        Key::U => Some(KeyCode::U),
        Key::V => Some(KeyCode::V),
        Key::W => Some(KeyCode::W),
        Key::X => Some(KeyCode::X),
        Key::Y => Some(KeyCode::Y),
        Key::Z => Some(KeyCode::Z),
        Key::F1 => Some(KeyCode::F1),
        Key::F2 => Some(KeyCode::F2),
        Key::F3 => Some(KeyCode::F3),
        Key::F4 => Some(KeyCode::F4),
        Key::F5 => Some(KeyCode::F5),
        Key::F6 => Some(KeyCode::F6),
        Key::F7 => Some(KeyCode::F7),
        Key::F8 => Some(KeyCode::F8),
        Key::F9 => Some(KeyCode::F9),
        Key::F10 => Some(KeyCode::F10),
        Key::F11 => Some(KeyCode::F11),
        Key::F12 => Some(KeyCode::F12),
        Key::F13 => Some(KeyCode::F13),
        Key::F14 => Some(KeyCode::F14),
        Key::F15 => Some(KeyCode::F15),
        Key::F16 => Some(KeyCode::F16),
        Key::F17 => Some(KeyCode::F17),
        Key::F18 => Some(KeyCode::F18),
        Key::F19 => Some(KeyCode::F19),
        Key::F20 => Some(KeyCode::F20),
        Key::F21 => Some(KeyCode::F21),
        Key::F22 => Some(KeyCode::F22),
        Key::F23 => Some(KeyCode::F23),
        Key::F24 => Some(KeyCode::F24),
        Key::F25 => None,
        Key::F26 => None,
        Key::F27 => None,
        Key::F28 => None,
        Key::F29 => None,
        Key::F30 => None,
        Key::F31 => None,
        Key::F32 => None,
        Key::F33 => None,
        Key::F34 => None,
        Key::F35 => None,
        Key::BrowserBack => None,
    }
}

fn covey_key_code_to_egui_key(kc: KeyCode) -> Key {
    match kc {
        KeyCode::Digit0 => Key::Num0,
        KeyCode::Digit1 => Key::Num1,
        KeyCode::Digit2 => Key::Num2,
        KeyCode::Digit3 => Key::Num3,
        KeyCode::Digit4 => Key::Num4,
        KeyCode::Digit5 => Key::Num5,
        KeyCode::Digit6 => Key::Num6,
        KeyCode::Digit7 => Key::Num7,
        KeyCode::Digit8 => Key::Num8,
        KeyCode::Digit9 => Key::Num9,
        KeyCode::A => Key::A,
        KeyCode::B => Key::B,
        KeyCode::C => Key::C,
        KeyCode::D => Key::D,
        KeyCode::E => Key::E,
        KeyCode::F => Key::F,
        KeyCode::G => Key::G,
        KeyCode::H => Key::H,
        KeyCode::I => Key::I,
        KeyCode::J => Key::J,
        KeyCode::K => Key::K,
        KeyCode::L => Key::L,
        KeyCode::M => Key::M,
        KeyCode::N => Key::N,
        KeyCode::O => Key::O,
        KeyCode::P => Key::P,
        KeyCode::Q => Key::Q,
        KeyCode::R => Key::R,
        KeyCode::S => Key::S,
        KeyCode::T => Key::T,
        KeyCode::U => Key::U,
        KeyCode::V => Key::V,
        KeyCode::W => Key::W,
        KeyCode::X => Key::X,
        KeyCode::Y => Key::Y,
        KeyCode::Z => Key::Z,
        KeyCode::F1 => Key::F1,
        KeyCode::F2 => Key::F2,
        KeyCode::F3 => Key::F3,
        KeyCode::F4 => Key::F4,
        KeyCode::F5 => Key::F5,
        KeyCode::F6 => Key::F6,
        KeyCode::F7 => Key::F7,
        KeyCode::F8 => Key::F8,
        KeyCode::F9 => Key::F9,
        KeyCode::F10 => Key::F10,
        KeyCode::F11 => Key::F11,
        KeyCode::F12 => Key::F12,
        KeyCode::F13 => Key::F13,
        KeyCode::F14 => Key::F14,
        KeyCode::F15 => Key::F15,
        KeyCode::F16 => Key::F16,
        KeyCode::F17 => Key::F17,
        KeyCode::F18 => Key::F18,
        KeyCode::F19 => Key::F19,
        KeyCode::F20 => Key::F20,
        KeyCode::F21 => Key::F21,
        KeyCode::F22 => Key::F22,
        KeyCode::F23 => Key::F23,
        KeyCode::F24 => Key::F24,
        KeyCode::Backtick => Key::Backtick,
        KeyCode::Hyphen => Key::Minus,
        KeyCode::Equal => Key::Equals,
        KeyCode::Tab => Key::Tab,
        KeyCode::LeftBracket => Key::OpenBracket,
        KeyCode::RightBracket => Key::CloseBracket,
        KeyCode::Backslash => Key::Backslash,
        KeyCode::Semicolon => Key::Semicolon,
        KeyCode::Apostrophe => Key::Quote,
        KeyCode::Enter => Key::Enter,
        KeyCode::Comma => Key::Comma,
        KeyCode::Period => Key::Period,
        KeyCode::Slash => Key::Slash,
    }
}
