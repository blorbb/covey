use color_eyre::eyre::Result;
use qpmu::{plugin::Plugin, Details};
use relm4::{
    gtk::{self, prelude::GtkWindowExt},
    Component, RelmContainerExt,
};

use crate::model::LauncherMsg;

#[derive(Debug)]
pub struct Settings {
    plugins: Vec<Plugin>,
}

#[derive(Debug)]
pub struct SettingsWidgets {
    plugin_list: gtk::Box,
}

#[derive(Debug)]
pub enum SettingsMsg {
    Show,
}

#[derive(Debug)]
pub enum SettingsCmd {
    ListDetails(Vec<Result<Details>>),
}

#[derive(Debug)]
pub enum SettingsOutput {}

impl Component for Settings {
    type CommandOutput = SettingsCmd;
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
        init: Self::Init,
        root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        {
            let init = init.clone();
            sender.oneshot_command(async move {
                let mut v = vec![];
                for plugin in init {
                    v.push(plugin.details().await);
                }
                SettingsCmd::ListDetails(v)
            });
        }

        let list = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .build();
        root.container_add(&list);

        relm4::ComponentParts {
            model: Self { plugins: init },
            widgets: SettingsWidgets { plugin_list: list },
        }
    }

    fn update(
        &mut self,
        message: Self::Input,
        sender: relm4::ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match message {
            SettingsMsg::Show => root.present(),
        }
    }

    fn update_cmd_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::CommandOutput,
        sender: relm4::ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match message {
            SettingsCmd::ListDetails(vec) => {
                for detail in vec {
                    widgets
                        .plugin_list
                        .container_add(&gtk::Label::new(Some(&format!("{:#?}", detail))));
                }
            }
        }
    }
}

pub fn output_transform(input: SettingsOutput) -> LauncherMsg {
    match input {}
}
