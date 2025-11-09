use covey::host;
use egui::{Key, Modifiers, ScrollArea, TextEdit, Ui, style::ScrollAnimation, text::CCursor};

pub mod cli;

pub struct App {
    cli: cli::Receiver,
    tx: host::RequestSender,
    rx: host::ResponseReceiver,
    input: String,
    list: Option<covey::List>,
    list_selection: usize,
}

impl App {
    pub fn new(cli_rx: &cli::Receiver) -> anyhow::Result<Self> {
        let (tx, rx) = covey::host::channel()?;
        Ok(Self {
            cli: cli_rx.clone(),
            tx,
            rx,
            input: String::new(),
            list: None,
            list_selection: 0,
        })
    }

    /// Open the app once, returning when it closes.
    pub fn open(&mut self) -> eframe::Result {
        let options = eframe::NativeOptions {
            run_and_return: true,
            viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
            ..Default::default()
        };

        eframe::run_native("covey", options.clone(), Box::new(|_cc| Ok(Box::new(self))))
    }
}

impl eframe::App for &mut App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // CLI window actions //
            match self.cli.try_recv() {
                Some(cli::Message::Exit) => {
                    tracing::info!("received exit message");
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close)
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
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
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
            } else if key_pressed_consume(ui, Key::ArrowDown) {
                if let Some(list) = &self.list {
                    self.list_selection = bounded_wrapping_add(self.list_selection, 1, list.len());
                    list_selection_changed = true;
                }
            } else if key_pressed_consume(ui, Key::ArrowUp) {
                if let Some(list) = &self.list {
                    self.list_selection = bounded_wrapping_sub(self.list_selection, 1, list.len());
                    list_selection_changed = true;
                }
            }

            // the actual UI //

            // text edit
            let mut text_edit = TextEdit::singleline(&mut self.input)
                .hint_text("Search...")
                .desired_width(f32::INFINITY)
                .return_key(None)
                .show(ui);

            if let Some((min, max)) = new_selection {
                text_edit.cursor_range = Some(egui::text::CCursorRange::two(
                    CCursor::new(min),
                    CCursor::new(max),
                ))
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
                ScrollArea::vertical().show(ui, |ui| {
                    for (i, item) in list.items.iter().enumerate() {
                        let response = ui.radio_value(&mut self.list_selection, i, item.title());
                        // Can't use response.changed() as that
                        // doesn't detect changes to self.selection
                        if self.list_selection == i && list_selection_changed {
                            response.scroll_to_me_animation(None, ScrollAnimation::duration(0.2));
                        }
                    }
                });
            }
        });
    }
}

fn key_pressed_consume(ui: &mut Ui, key: Key) -> bool {
    ui.input_mut(|state| state.consume_key(Modifiers::NONE, key))
}

fn bounded_wrapping_add(x: usize, amount: usize, max_excl: usize) -> usize {
    (x + amount) % max_excl
}

fn bounded_wrapping_sub(x: usize, amount: usize, max_excl: usize) -> usize {
    (x + max_excl - (amount % max_excl)) % max_excl
}
