pub mod cli;

pub struct App {
    rx: cli::Receiver,
}

impl App {
    pub fn new(rx: &cli::Receiver) -> Self {
        Self { rx: rx.clone() }
    }

    /// Open the app once, returning when it closes.
    pub fn open(rx: &cli::Receiver) -> eframe::Result {
        let options = eframe::NativeOptions {
            run_and_return: true,
            viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
            ..Default::default()
        };

        eframe::run_native(
            "covey",
            options.clone(),
            Box::new(|_cc| Ok(Box::new(Self::new(&rx)))),
        )
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.rx.try_recv() {
                Some(cli::Message::Exit) => {
                    tracing::info!("received exit message");
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close)
                }
                // Trying to open while already open -> do nothing
                Some(cli::Message::Open) => {}
                None => {}
            }

            let label_text =
                "When this window is closed the next will be opened after a short delay";
            ui.label(label_text);

            // https://github.com/emilk/egui/issues/3655#issuecomment-3239209608
            // TODO: try the above
            if ui.button("Close").clicked() {
                tracing::info!("Pressed Close button");
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
    }
}
