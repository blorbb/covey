use covey::{ListItem, covey_schema::style::UserStyle};
use egui::{Align, Layout, TextStyle, Ui, Vec2};

use crate::{
    AsEgui, ICON_TEXT_STYLE,
    widgets::{Container, ImageIcon},
};

pub struct ListCell<'sel, 'item, Value> {
    current_value: &'sel mut Value,
    selected_value: Value,
    item: &'item ListItem,
}

impl<'sel, 'item, Value: PartialEq> ListCell<'sel, 'item, Value> {
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

    pub fn show(
        self,
        ui: &mut Ui,
        style: &UserStyle,
        list_style: covey::ListStyle,
    ) -> egui::Response {
        let icon_text_layout = match list_style {
            covey::ListStyle::Rows => Layout::left_to_right(Align::Min),
            covey::ListStyle::Grid | covey::ListStyle::GridWithColumns(_) => {
                Layout::top_down(Align::Center)
            }
        };
        let title_desc_layout = match list_style {
            covey::ListStyle::Rows => Layout::top_down(Align::Min),
            covey::ListStyle::Grid | covey::ListStyle::GridWithColumns(_) => {
                Layout::top_down(Align::Center)
            }
        };

        let mut button = Container::new()
            .fill(style.list_item_bg().as_egui())
            .hover_fill(style.list_item_hovered_bg().as_egui())
            .active_fill(style.list_item_active_bg().as_egui())
            .inner_margin(style.list_item_padding().as_egui())
            .corner_radius(style.list_item_rounding().into())
            .selected(*self.current_value == self.selected_value)
            .min_size(Vec2::new(ui.available_width(), 0.0))
            .show_with_layout(ui, icon_text_layout, |ui| {
                ui.spacing_mut().item_spacing = style.list_item_padding().as_egui();

                if let Some(icon) = self.item.icon() {
                    ui.add(CellIcon::new(icon.clone()));
                }

                ui.with_layout(title_desc_layout, |ui| {
                    ui.spacing_mut().item_spacing = Vec2::ZERO;

                    ui.label(self.item.title());

                    if !self.item.description().is_empty() {
                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                        ui.label(
                            egui::RichText::new(self.item.description())
                                .font(TextStyle::Small.resolve(ui.style()))
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

struct CellIcon {
    icon: covey::ResolvedIcon,
}

impl CellIcon {
    fn new(icon: covey::ResolvedIcon) -> Self {
        Self { icon }
    }
}

impl egui::Widget for CellIcon {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let icon_size = Vec2::splat(crate::icon_size(ui.ctx()));

        match self.icon {
            covey::ResolvedIcon::File(file_path) => {
                ui.add(ImageIcon::from_file_path(file_path, icon_size))
            }
            covey::ResolvedIcon::Text(text) => ui.add_sized(
                icon_size,
                egui::Label::new(
                    egui::RichText::new(text).font(ICON_TEXT_STYLE.resolve(ui.style())),
                ),
            ),
        }
    }
}
