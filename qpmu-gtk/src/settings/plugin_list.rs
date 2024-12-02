use az::{CheckedAs, SaturatingAs};
use qpmu::plugin::Plugin;
use relm4::{
    gtk::{
        self,
        prelude::{BoxExt, ButtonExt, ListBoxRowExt},
        ListBox,
    },
    Component, ComponentParts,
};
use tracing::info;

#[derive(Debug)]
pub struct PluginList {
    plugins: Vec<Plugin>,
}

#[derive(Debug)]
pub enum Msg {
    Focus { index: usize },
    Move { index: usize, delta: isize },
}

#[derive(Debug)]
pub enum Output {
    SetPluginList(Vec<Plugin>),
}

#[derive(Debug)]
pub struct Widgets {
    items: ListBox,
}

impl Component for PluginList {
    type Input = Msg;
    type Output = Output;
    type CommandOutput = ();
    type Init = Vec<Plugin>;
    type Root = gtk::ListBox;
    type Widgets = Widgets;

    fn init_root() -> Self::Root {
        gtk::ListBox::builder()
            .css_classes(["plugin-list-wrapper"])
            .selection_mode(gtk::SelectionMode::Single)
            .build()
    }

    fn init(
        plugins: Self::Init,
        root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = Self {
            plugins: plugins.clone(),
        };

        for plugin in plugins {
            root.append(&create_list_row(&plugin, &sender));
        }

        root.connect_row_activated(|_, row| {
            info!("todo activated {}", row.index());
        });

        ComponentParts {
            model,
            widgets: Widgets { items: root },
        }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: relm4::ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match message {
            Msg::Focus { index } => info!("todo activated {index}"),
            Msg::Move { index, delta } => {
                if self.plugins.len() <= index {
                    return;
                }
                let new_index = usize::min(
                    index.saturating_add_signed(delta),
                    self.plugins.len().saturating_sub(1),
                );
                // shift the plugin to the new location
                if index < new_index {
                    // e.g. existing is 'a'
                    // [a, b, c] -> [b, c, a]
                    // rotate left
                    self.plugins[index..=new_index].rotate_left(1);
                } else {
                    // e.g. existing is 'c'
                    // [a, b, c] -> [c, a, b]
                    // rotate right
                    self.plugins[new_index..=index].rotate_right(1);
                }
                dbg!(&self.plugins);

                let target = widgets
                    .items
                    .row_at_index(index.saturating_as::<i32>())
                    .expect("length checked");
                widgets.items.remove(&target);
                widgets
                    .items
                    .insert(&target, new_index.saturating_as::<i32>());

                sender
                    .output(Output::SetPluginList(self.plugins.clone()))
                    .expect("parent should not be detached");
            }
        }
    }
}

fn create_list_row(
    plugin: &Plugin,
    sender: &relm4::ComponentSender<PluginList>,
) -> gtk::ListBoxRow {
    let list_row = gtk::ListBoxRow::new();
    let inner_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .build();

    let label = gtk::Label::builder()
        .label(plugin.name())
        .halign(gtk::Align::Start)
        .build();

    let spacer = gtk::Box::builder().hexpand(true).build();

    let up_button = gtk::Button::builder()
        .child(&gtk::Image::from_icon_name("up"))
        .build();
    up_button.connect_clicked({
        let sender = sender.clone();
        let list_row = list_row.clone();
        move |_| {
            let item_index = list_row
                .index()
                .checked_as::<usize>()
                .expect("index should not be negative");
            sender.input(Msg::Move {
                index: item_index,
                delta: -1,
            });
        }
    });

    let down_button = gtk::Button::builder()
        .child(&gtk::Image::from_icon_name("down"))
        .build();
    down_button.connect_clicked({
        let sender = sender.clone();
        let list_row = list_row.clone();
        move |_| {
            let item_index = list_row
                .index()
                .checked_as::<usize>()
                .expect("index should not be negative");
            sender.input(Msg::Move {
                index: item_index,
                delta: 1,
            });
        }
    });

    inner_box.append(&label);
    inner_box.append(&spacer);
    inner_box.append(&up_button);
    inner_box.append(&down_button);
    list_row.set_child(Some(&inner_box));

    list_row
}
