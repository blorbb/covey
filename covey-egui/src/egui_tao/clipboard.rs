use raw_window_handle::RawDisplayHandle;

/// Handles interfacing with the OS clipboard.
///
/// If the "clipboard" feature is off, or we cannot connect to the OS clipboard,
/// then a fallback clipboard that just works within the same app is used instead.
pub struct Clipboard {
    arboard: Option<arboard::Clipboard>,
    /// Fallback manual clipboard.
    clipboard: String,
}

impl Clipboard {
    /// Construct a new instance
    pub fn new(_raw_display_handle: Option<RawDisplayHandle>) -> Self {
        Self {
            arboard: init_arboard(),
            clipboard: Default::default(),
        }
    }

    pub fn get(&mut self) -> Option<String> {
        if let Some(clipboard) = &mut self.arboard {
            return match clipboard.get_text() {
                Ok(text) => Some(text),
                Err(err) => {
                    tracing::error!("arboard paste error: {err}");
                    None
                }
            };
        }

        Some(self.clipboard.clone())
    }

    pub fn set_text(&mut self, text: String) {
        if let Some(clipboard) = &mut self.arboard {
            if let Err(err) = clipboard.set_text(text) {
                tracing::error!("arboard copy/cut error: {err}");
            }
            return;
        }

        self.clipboard = text;
    }

    pub fn set_image(&mut self, image: &egui::ColorImage) {
        if let Some(clipboard) = &mut self.arboard {
            if let Err(err) = clipboard.set_image(arboard::ImageData {
                width: image.width(),
                height: image.height(),
                bytes: std::borrow::Cow::Borrowed(bytemuck::cast_slice(&image.pixels)),
            }) {
                tracing::error!("arboard copy/cut error: {err}");
            }
            tracing::debug!("Copied image to clipboard");
            return;
        }

        tracing::error!(
            "Copying images is not supported. Enable the 'clipboard' feature of `egui-winit` to enable it."
        );
        _ = image;
    }
}

fn init_arboard() -> Option<arboard::Clipboard> {
    profiling::function_scope!();

    tracing::trace!("Initializing arboard clipboard…");
    match arboard::Clipboard::new() {
        Ok(clipboard) => Some(clipboard),
        Err(err) => {
            tracing::warn!("Failed to initialize arboard clipboard: {err}");
            None
        }
    }
}
