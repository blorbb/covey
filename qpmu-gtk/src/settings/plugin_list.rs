use az::SaturatingAs;
use qpmu::plugin::Plugin;
use relm4::{
    gtk::{
        self,
        prelude::{BoxExt, ButtonExt, ListBoxRowExt},
        ListBox,
    },
    Component, ComponentParts, RelmContainerExt,
};
use tracing::{debug, info};

#[derive(Debug)]
pub struct PluginList {
    plugins: Vec<Plugin>,
    selected: Option<usize>,
}

#[derive(Debug)]
pub enum Msg {
    Focus(usize),
    MoveSelected { delta: isize },
}

#[derive(Debug)]
pub enum Output {
    SetPluginList(Vec<Plugin>),
    SetSelection(usize),
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
    type Root = gtk::Box;
    type Widgets = Widgets;

    fn init_root() -> Self::Root {
        gtk::Box::builder()
            .css_classes(["plugin-list-wrapper"])
            .orientation(gtk::Orientation::Vertical)
            .build()
    }

    fn init(
        plugins: Self::Init,
        root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = Self {
            plugins: plugins.clone(),
            selected: None,
        };

        let list = gtk::ListBox::builder()
            .css_classes(["plugin-list"])
            .selection_mode(gtk::SelectionMode::Single)
            .build();

        for plugin in plugins {
            list.append(&create_list_row(&plugin));
        }

        list.connect_row_activated({
            let sender = sender.clone();
            move |_, row| {
                info!("activated row {}", row.index());
                sender.input(Msg::Focus(
                    row.index()
                        .try_into()
                        .expect("index should be non-negative"),
                ));
            }
        });

        let controls = gtk::Box::builder()
            .css_classes(["plugin-list-controls"])
            .orientation(gtk::Orientation::Horizontal)
            .build();

        let up_button = gtk::Button::builder()
            .child(&gtk::Image::from_icon_name("up"))
            .build();
        up_button.connect_clicked({
            let sender = sender.clone();
            move |_| {
                debug!("clicked up button");
                sender.input(Msg::MoveSelected { delta: -1 });
            }
        });

        let down_button = gtk::Button::builder()
            .child(&gtk::Image::from_icon_name("down"))
            .build();
        down_button.connect_clicked({
            let sender = sender.clone();
            move |_| {
                debug!("clicked down button");
                sender.input(Msg::MoveSelected { delta: 1 });
            }
        });

        controls.container_add(&up_button);
        controls.container_add(&down_button);

        root.container_add(&list);
        root.container_add(&controls);

        ComponentParts {
            model,
            widgets: Widgets { items: list },
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
            Msg::Focus(index) => {
                debug!("focusing row {index}");
                self.selected = Some(index);
                sender.output(Output::SetSelection(index)).unwrap()
            }
            Msg::MoveSelected { delta } => {
                if let Some(index) = self.selected {
                    debug!("moving {index} by {delta}");
                    self.move_plugin(index, delta, widgets, sender);
                } else {
                    debug!("no existing selection");
                }
            }
        }
    }
}

impl PluginList {
    fn move_plugin(
        &mut self,
        index: usize,
        delta: isize,
        widgets: &mut Widgets,
        sender: relm4::ComponentSender<Self>,
    ) {
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

        let target = widgets
            .items
            .row_at_index(index.saturating_as::<i32>())
            .expect("length checked");
        widgets.items.remove(&target);
        widgets
            .items
            .insert(&target, new_index.saturating_as::<i32>());

        self.selected = Some(new_index);
        sender
            .output(Output::SetPluginList(self.plugins.clone()))
            .unwrap();
        sender.output(Output::SetSelection(new_index)).unwrap();
    }
}

fn create_list_row(plugin: &Plugin) -> gtk::ListBoxRow {
    let list_row = gtk::ListBoxRow::new();
    // add more stuff into this inner box later
    let inner_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .build();

    let label = gtk::Label::builder()
        .label(plugin.name())
        .halign(gtk::Align::Start)
        .build();

    inner_box.append(&label);
    list_row.set_child(Some(&inner_box));

    list_row
}
