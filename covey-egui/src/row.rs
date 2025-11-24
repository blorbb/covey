use covey::ListItem;
use egui::{Atom, AtomExt, Button, TextFormat, TextStyle, Ui, Vec2, text::LayoutJob};

use crate::ICON_TEXT_STYLE;

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
}

impl<Value: PartialEq> egui::Widget for ListRow<'_, '_, Value> {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let selected = *self.current_value == self.selected_value;

        let body_font = TextStyle::Body.resolve(ui.style());
        let desc_font = TextStyle::Small.resolve(ui.style());
        let icon_font = ICON_TEXT_STYLE.resolve(ui.style());
        let icon_size = Vec2::splat(icon_font.size);

        let icon = match self.item.icon() {
            Some(covey::ResolvedIcon::File(file_path)) => {
                let image = egui::Image::new(format!(
                    "file://{}",
                    file_path.as_os_str().to_string_lossy()
                ));
                image.fit_to_exact_size(icon_size).atom_size(icon_size)
            }
            Some(covey::ResolvedIcon::Text(_text)) => {
                // might need to use egui_taffy for this, more complex layout within a button
                // can't seem to get centered text working with LayoutJob / Galley
                Atom::default()
            }
            None => Atom::default(),
        };

        let mut text = LayoutJob::default();
        text.append(
            &self.item.title(),
            0.0,
            TextFormat::simple(body_font, ui.style().visuals.text_color()),
        );

        if !self.item.description().is_empty() {
            text.append("\n", 0.0, TextFormat::default());
            text.append(
                &self.item.description(),
                0.0,
                TextFormat::simple(desc_font, ui.style().visuals.weak_text_color()),
            );
        }

        let mut button = Button::new((icon, text))
            .min_size(Vec2::new(ui.available_width(), 0.0))
            .selected(selected)
            .ui(ui);

        if button.clicked() {
            *self.current_value = self.selected_value;
            button.mark_changed();
            dbg!(button.changed());
        }

        button
    }
}
