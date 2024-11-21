use color_eyre::eyre::Result;
use relm4::ComponentSender;
use tray_item::TrayItem;

use crate::model::{Launcher, LauncherMsg};

pub fn create_tray_icon(sender: ComponentSender<Launcher>) -> Result<()> {
    // TODO: get a proper icon
    let mut tray = TrayItem::new("qpmu", tray_item::IconSource::Resource("application-menu"))?;
    tray.add_menu_item("Quit", {
        let sender = sender.clone();
        move || {
            sender.input(LauncherMsg::Shutdown);
        }
    })?;

    Ok(())
}
