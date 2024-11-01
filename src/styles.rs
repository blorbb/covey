use color_eyre::eyre::{ContextCompat, Result};
use gtk::{gdk::Display, CssProvider};

pub fn load_css() -> Result<()> {
    let css = CssProvider::new();
    css.load_from_string(include_str!("../styles/style.css"));
    gtk::style_context_add_provider_for_display(
        &Display::default().context("could not connect to a display")?,
        &css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    Ok(())
}
