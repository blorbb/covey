use az::CheckedAs;
use qpmu::{config::Config, plugin::Plugin, BoundedUsize, CONFIG_PATH};
use relm4::{
    gtk::{
        self,
        prelude::{GridExt, GtkWindowExt, ListBoxRowExt},
    },
    Component, Controller, RelmContainerExt,
};

use super::{
    messages::{SettingsMsg, SettingsOutput},
    plugin_list::PluginList,
};

#[derive(Debug)]
pub struct Settings {
    plugins: Vec<Plugin>,
    selected_plugin: Option<BoundedUsize>,
    config: Config,
    plugin_list_component: Controller<PluginList>,
}

impl Settings {
    /// Writes the contents of the TOML document to the config file.
    ///
    /// This should be called after mutating [`Self::document`].
    fn update_document(config: &Config, sender: &relm4::ComponentSender<Self>) {
        let config_toml =
            toml::to_string_pretty(config).expect("failed to serialise configuration");
        std::fs::write(&*CONFIG_PATH, config_toml).expect("failed to write to config file");
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
                config: toml::from_str(
                    &std::fs::read_to_string(&*CONFIG_PATH).expect("failed to read qpmu config"),
                )
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
                self.config.reorder_plugins(&self.plugins);
                Self::update_document(&self.config, &sender);
            }
        }
    }
}
