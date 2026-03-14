use egui::{
    Color32, CornerRadius, InnerResponse, Margin, NumExt, Sense, Stroke, Ui, Vec2,
    epaint::MarginF32,
};

// maybe make the hover and active states just more frames?
// margins cannot change after the frame is allocated though, which is when the
// state of the button is known.

pub struct ButtonStyle {
    frame: egui::Frame,
    // `None` means use the existing frame.
    hover_fill: Option<Color32>,
    hover_stroke: Option<Stroke>,
    // `None` means use the hover style.
    // "active" is when the button is being pressed / focused / selected.
    active_fill: Option<Color32>,
    active_stroke: Option<Stroke>,
    min_size: Vec2,
}

pub struct ButtonFrame {
    style: ButtonStyle,
    selected: bool,
}

impl ButtonFrame {
    pub fn new() -> Self {
        Self {
            style: ButtonStyle {
                frame: egui::Frame::new(),
                hover_fill: None,
                hover_stroke: None,
                active_fill: None,
                active_stroke: None,
                min_size: Vec2::ZERO,
            },
            selected: false,
        }
    }

    // /// By default, buttons senses clicks.
    // /// Change this to a drag-button with `Sense::drag()`.
    // #[inline]
    // pub fn sense(mut self, sense: Sense) -> Self {
    //     self.layout = self.layout.sense(sense);
    //     self
    // }

    #[inline]
    pub fn min_size(mut self, min_size: Vec2) -> Self {
        self.style.min_size = min_size;
        self
    }

    pub fn inner_margin(mut self, margin: Margin) -> Self {
        self.style.frame.inner_margin = margin;
        self
    }

    pub fn outer_margin(mut self, margin: Margin) -> Self {
        self.style.frame.outer_margin = margin;
        self
    }

    pub fn corner_radius(mut self, corner_radius: CornerRadius) -> Self {
        self.style.frame.corner_radius = corner_radius;
        self
    }

    #[inline]
    pub fn fill(mut self, fill: Color32) -> Self {
        self.style.frame.fill = fill;
        self
    }

    #[inline]
    pub fn stroke(mut self, stroke: Stroke) -> Self {
        self.style.frame.stroke = stroke;
        self
    }

    #[inline]
    pub fn hover_fill(mut self, fill: Color32) -> Self {
        self.style.hover_fill = Some(fill);
        self
    }

    #[inline]
    pub fn hover_stroke(mut self, stroke: Stroke) -> Self {
        self.style.hover_stroke = Some(stroke);
        self
    }

    #[inline]
    pub fn active_fill(mut self, fill: Color32) -> Self {
        self.style.active_fill = Some(fill);
        self
    }

    #[inline]
    pub fn active_stroke(mut self, stroke: Stroke) -> Self {
        self.style.active_stroke = Some(stroke);
        self
    }

    #[inline]
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
        frame.content_ui.set_min_size(
            (self.style.min_size
                - frame.frame.inner_margin.sum()
                - Vec2::splat(frame.frame.stroke.width)
                - frame.frame.outer_margin.sum())
            .at_least(Vec2::ZERO),
        );
        let inner_response = add_contents(&mut frame.content_ui);

        // allocate_space only supports sensing hovers and can't be customized.
        // this is a copy of allocate_space but with extra senses.
        // also the min size.
        let outer_rect = frame.content_ui.min_rect()
            + frame.frame.inner_margin
            + MarginF32::from(frame.frame.stroke.width)
            + frame.frame.outer_margin;
        let response = ui.allocate_rect(outer_rect, Sense::click());

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
