use covey::ListItem;
use egui::{Button, TextFormat, TextStyle, Ui, Vec2, text::LayoutJob};

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

        let mut text = LayoutJob::default();
        text.append(
            &self.item.title(),
            0.0,
            TextFormat::simple(
                TextStyle::Body.resolve(ui.style()),
                ui.style().visuals.text_color(),
            ),
        );

        if !self.item.description().is_empty() {
            text.append("\n", 0.0, TextFormat::default());
            text.append(
                &self.item.description(),
                0.0,
                TextFormat::simple(
                    TextStyle::Small.resolve(ui.style()),
                    ui.style().visuals.weak_text_color(),
                ),
            );
        }

        let mut button = Button::new(text)
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
