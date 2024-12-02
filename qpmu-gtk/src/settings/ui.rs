use std::mem;

use az::CheckedAs;
use qpmu::{plugin::Plugin, BoundedUsize, CONFIG_PATH};
use relm4::{
    gtk::{
        self,
        prelude::{GridExt, GtkWindowExt, ListBoxRowExt},
    },
    Component, Controller, RelmContainerExt,
};
use tap::Tap;
use toml_edit::{ArrayOfTables, DocumentMut, Item, Table};

use super::{
    messages::{SettingsMsg, SettingsOutput},
    plugin_list::PluginList,
};

#[derive(Debug)]
pub struct Settings {
    plugins: Vec<Plugin>,
    selected_plugin: Option<BoundedUsize>,
    document: DocumentMut,
    plugin_list_component: Controller<PluginList>,
}

impl Settings {
    /// Writes the contents of the TOML document to the config file.
    ///
    /// This should be called after mutating [`Self::document`].
    fn update_document(&self, sender: &relm4::ComponentSender<Self>) {
        std::fs::write(&*CONFIG_PATH, self.document.to_string())
            .expect("failed to write to config file");
        sender
            .output(SettingsOutput::ReloadPlugins)
            .expect("parent receiver should not be dropped");
    }
}

#[derive(Debug)]
pub struct SettingsWidgets {
    layout: gtk::Grid,
}

impl Component for Settings {
    type CommandOutput = ();
    type Input = SettingsMsg;
    type Output = SettingsOutput;
    type Init = Vec<Plugin>;
    type Root = gtk::Window;
    type Widgets = SettingsWidgets;

    fn init_root() -> Self::Root {
        gtk::Window::builder()
            .css_classes(["settings-window"])
            .build()
    }

    fn init(
        plugins: Self::Init,
        root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let plugin_list = gtk::ListBox::builder()
            .css_classes(["plugin-list"])
            .selection_mode(gtk::SelectionMode::Single)
            .build();

        plugin_list.connect_row_activated({
            let sender = sender.clone();
            move |_, row| {
                let selection = row
                    .index()
                    .checked_as::<usize>()
                    .expect("index should be non negative");
                sender.input(SettingsMsg::SetSelection(Some(selection)));
            }
        });

        let layout = gtk::Grid::new();
        let plugin_list_wrapper = gtk::Box::builder().build();
        layout.attach(&plugin_list_wrapper, 0, 0, 1, 1);
        let plugin_list = PluginList::builder()
            .attach_to(&plugin_list_wrapper)
            .launch(plugins.clone())
            .forward(sender.input_sender(), SettingsMsg::from);
        root.container_add(&layout);

        relm4::ComponentParts {
            model: Self {
                plugins,
                selected_plugin: None,
                document: std::fs::read_to_string(&*CONFIG_PATH)
                    .expect("failed to read qpmu config")
                    .parse::<DocumentMut>()
                    .expect("invalid configuration document"),
                plugin_list_component: plugin_list,
            },
            widgets: SettingsWidgets { layout },
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
            SettingsMsg::Show => root.present(),

            SettingsMsg::SetSelection(selection) => {
                self.selected_plugin = selection.and_then(|selection| {
                    let mut bounded =
                        BoundedUsize::new_with_bound(self.plugins.len().checked_sub(1)?);
                    bounded.saturating_set(selection);
                    Some(bounded)
                })
            }
            SettingsMsg::SetPluginList(plugins) => {
                self.plugins = plugins;
                eprintln!("new plugins are {:?}", self.plugins);

                let document = &mut self.document;
                // reorder the config file
                let toml_list = document
                    .entry("plugins")
                    .or_insert(Item::ArrayOfTables(ArrayOfTables::new()))
                    .as_array_of_tables_mut()
                    .expect("invalid config format: key 'plugins' should be an array of tables");

                // either find a table that matches each plugin name, or make a new one
                let new_array: ArrayOfTables = self
                    .plugins
                    .iter()
                    .map(|plugin| -> Table {
                        toml_list
                            .iter_mut()
                            .find_map(|table| {
                                (table.get("name")?.as_str()? == plugin.name())
                                    .then(|| mem::take(table))
                            })
                            .unwrap_or_else(|| {
                                Table::new().tap_mut(|table| {
                                    table.insert("name", plugin.name().into());
                                    table.insert("prefix", plugin.prefix().into());
                                })
                            })
                    })
                    // need to explicitly set the table's
                    // position or it doesn't work properly
                    // TODO: use toml instead of toml edit because this is still
                    // broken. can't just enumerate, need to figure out
                    // global positions, will be a mess.
                    .enumerate()
                    .map(|(i, mut table)| {
                        table.set_position(i);
                        table
                    })
                    .collect();
                *toml_list = new_array;
                self.update_document(&sender);
            }
        }
    }
}
