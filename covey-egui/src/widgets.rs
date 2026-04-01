use std::path::PathBuf;

use egui::{
    Color32, CornerRadius, Image, InnerResponse, Margin, NumExt, Sense, Stroke, Ui, Vec2, Widget,
};

// maybe make the hover and active states just more frames?
// margins cannot change after the frame is allocated though, which is when the
// state of the button is known.

pub struct ContainerStyle {
    frame: egui::Frame,
    // `None` means use the existing frame.
    hover_fill: Option<Color32>,
    hover_stroke: Option<Stroke>,
    // `None` means use the hover style.
    // "active" is when the button is being pressed / focused / selected.
    active_fill: Option<Color32>,
    active_stroke: Option<Stroke>,
    min_size: Vec2,
    max_size: Vec2,
}

pub struct Container {
    style: ContainerStyle,
    selected: bool,
    sense: Sense,
}

impl Container {
    pub fn new() -> Self {
        Self {
            style: ContainerStyle {
                frame: egui::Frame::new(),
                hover_fill: None,
                hover_stroke: None,
                active_fill: None,
                active_stroke: None,
                min_size: Vec2::ZERO,
                max_size: Vec2::INFINITY,
            },
            selected: false,
            sense: Sense::click(),
        }
    }

    /// By default, buttons senses clicks.
    /// Change this to a drag-button with `Sense::drag()`.
    #[must_use]
    pub fn sense(mut self, sense: Sense) -> Self {
        self.sense = sense;
        self
    }

    #[must_use]
    pub fn min_size(mut self, min_size: Vec2) -> Self {
        self.style.min_size = min_size;
        self
    }

    #[must_use]
    pub fn min_width(mut self, min_width: f32) -> Self {
        self.style.min_size.x = min_width;
        self
    }

    #[must_use]
    pub fn min_height(mut self, min_height: f32) -> Self {
        self.style.min_size.y = min_height;
        self
    }

    #[must_use]
    pub fn max_size(mut self, max_size: Vec2) -> Self {
        self.style.max_size = max_size;
        self
    }

    #[must_use]
    pub fn max_width(mut self, max_width: f32) -> Self {
        self.style.max_size.x = max_width;
        self
    }

    #[must_use]
    pub fn max_height(mut self, max_height: f32) -> Self {
        self.style.max_size.y = max_height;
        self
    }

    #[must_use]
    pub fn exact_size(self, size: Vec2) -> Self {
        self.min_size(size).max_size(size)
    }

    #[must_use]
    pub fn exact_width(self, width: f32) -> Self {
        self.min_width(width).max_width(width)
    }

    #[must_use]
    pub fn exact_height(self, height: f32) -> Self {
        self.min_height(height).max_height(height)
    }

    #[must_use]
    pub fn inner_margin(mut self, margin: Margin) -> Self {
        self.style.frame.inner_margin = margin;
        self
    }

    #[must_use]
    pub fn outer_margin(mut self, margin: Margin) -> Self {
        self.style.frame.outer_margin = margin;
        self
    }

    #[must_use]
    pub fn corner_radius(mut self, corner_radius: CornerRadius) -> Self {
        self.style.frame.corner_radius = corner_radius;
        self
    }

    #[must_use]
    pub fn fill(mut self, fill: Color32) -> Self {
        self.style.frame.fill = fill;
        self
    }

    #[must_use]
    pub fn stroke(mut self, stroke: Stroke) -> Self {
        self.style.frame.stroke = stroke;
        self
    }

    #[must_use]
    pub fn hover_fill(mut self, fill: Color32) -> Self {
        self.style.hover_fill = Some(fill);
        self
    }

    #[must_use]
    pub fn hover_stroke(mut self, stroke: Stroke) -> Self {
        self.style.hover_stroke = Some(stroke);
        self
    }

    #[must_use]
    pub fn active_fill(mut self, fill: Color32) -> Self {
        self.style.active_fill = Some(fill);
        self
    }

    #[must_use]
    pub fn active_stroke(mut self, stroke: Stroke) -> Self {
        self.style.active_stroke = Some(stroke);
        self
    }

    #[must_use]
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        self.show_dyn(ui, Box::new(add_contents))
    }

    pub fn show_with_layout<R>(
        self,
        ui: &mut Ui,
        layout: egui::Layout,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        self.show(ui, |ui| ui.with_layout(layout, add_contents).inner)
    }

    pub fn show_horizontal<R>(
        self,
        ui: &mut Ui,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        self.show(ui, |ui| ui.horizontal(add_contents).inner)
    }

    pub fn show_dyn<'c, R>(
        self,
        ui: &mut Ui,
        add_contents: Box<dyn FnOnce(&mut Ui) -> R + 'c>,
    ) -> InnerResponse<R> {
        let mut frame = self.style.frame.begin(ui);

        // must subtract the same amount that is added to the rect below
        let min_size =
            (self.style.min_size - frame.frame.total_margin().sum()).at_least(Vec2::ZERO);
        frame.content_ui.set_min_size(min_size);

        // egui doesn't handle infinities well
        let max_size =
            (self.style.max_size - frame.frame.total_margin().sum()).at_least(Vec2::ZERO);
        if max_size.x.is_finite() {
            frame.content_ui.set_max_width(max_size.x);
        }
        if max_size.y.is_finite() {
            frame.content_ui.set_max_height(max_size.y);
        }

        let inner_response = add_contents(&mut frame.content_ui);

        // allocate_space only supports sensing hovers and can't be customized.
        // this is a copy of allocate_space but with extra senses.
        // also the min size.
        let outer_rect = frame.content_ui.min_rect() + frame.frame.total_margin();
        let response = ui.allocate_rect(outer_rect, self.sense);

        if response.has_focus() || response.is_pointer_button_down_on() || self.selected {
            frame.frame.fill = self
                .style
                .active_fill
                .or(self.style.hover_fill)
                .unwrap_or(self.style.frame.fill);
            frame.frame.stroke = self
                .style
                .active_stroke
                .or(self.style.hover_stroke)
                .unwrap_or(self.style.frame.stroke);
        } else if response.hovered() {
            frame.frame.fill = self.style.hover_fill.unwrap_or(self.style.frame.fill);
            frame.frame.stroke = self.style.hover_stroke.unwrap_or(self.style.frame.stroke);
        }

        frame.paint(ui);

        InnerResponse {
            inner: inner_response,
            response,
        }
    }
}

pub struct ImageIcon {
    file_path: PathBuf,
    size: Vec2,
}

impl ImageIcon {
    pub fn from_file_path(file_path: PathBuf, size: Vec2) -> Self {
        Self { file_path, size }
    }

    pub fn from_icon_name(host: &covey::Host, icon_name: &str, size: Vec2) -> Option<Self> {
        Some(Self::from_file_path(
            covey::ResolvedIcon::resolve_icon_name(host, icon_name)?,
            size,
        ))
    }
}

impl Widget for ImageIcon {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        ui.add_sized(
            self.size,
            Image::new(format!("file://{}", self.file_path.display())).fit_to_exact_size(self.size),
        )
    }
}
