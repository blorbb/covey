use covey::{ListItem, covey_schema::style::UserStyle};
use egui::{TextStyle, Ui, Vec2};

use crate::{AsEgui, ICON_TEXT_STYLE, button::ButtonFrame};

pub struct ListRow<'sel, 'item, Value> {
    current_value: &'sel mut Value,
    selected_value: Value,
    item: &'item ListItem,
}

impl<'sel, 'item, Value: PartialEq> ListRow<'sel, 'item, Value> {
    pub fn new(
        current_value: &'sel mut Value,
        selected_value: Value,
        item: &'item ListItem,
    ) -> Self {
        Self {
            current_value,
            selected_value,
            item,
        }
    }

    pub fn show(self, ui: &mut Ui, style: &UserStyle) -> egui::Response {
        let mut button = ButtonFrame::new()
            .fill(style.list_item_bg().as_egui())
            .hover_fill(style.list_item_hovered_bg().as_egui())
            .active_fill(style.list_item_active_bg().as_egui())
            .inner_margin(style.list_item_padding().as_egui())
            .corner_radius(style.list_item_rounding().into())
            .selected(*self.current_value == self.selected_value)
            .min_size(Vec2::new(ui.available_width(), 0.0))
            .show_horizontal(ui, |ui| {
                ui.spacing_mut().item_spacing = style.list_item_padding().as_egui();

                // let body_font = TextStyle::Body.resolve(ui.style());
                let desc_font = TextStyle::Small.resolve(ui.style());
                let icon_font = ICON_TEXT_STYLE.resolve(ui.style());
                let icon_size = Vec2::splat(ui.fonts_mut(|f| f.row_height(&icon_font)));

                match self.item.icon() {
                    Some(covey::ResolvedIcon::File(file_path)) => {
                        let image = egui::Image::new(format!(
                            "file://{}",
                            file_path.as_os_str().to_string_lossy()
                        ));
                        ui.add_sized(icon_size, image.fit_to_exact_size(icon_size));
                    }
                    Some(covey::ResolvedIcon::Text(text)) => {
                        ui.add_sized(
                            icon_size,
                            egui::Label::new(egui::RichText::new(text).font(icon_font)),
                        );
                    }
                    None => {}
                };

                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::ZERO;
                    ui.label(self.item.title());
                    if !self.item.description().is_empty() {
                        ui.label(
                            egui::RichText::new(self.item.description())
                                .font(desc_font)
                                .color(style.weak_text_color().as_egui()),
                        );
                    }
                })
            })
            .response;

        if button.clicked() {
            *self.current_value = self.selected_value;
            button.mark_changed();
            dbg!(button.changed());
        }

        button
    }
}
