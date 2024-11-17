use qpmu::CONFIG_DIR;
use relm4::gtk::{self, gdk::Display, CssProvider};
use tracing::instrument;


#[instrument]
pub fn load_css() {
    let display = Display::default().expect("could not connect to a display");

    let default_css = CssProvider::new();
    default_css.load_from_data(include_str!("../styles/style.css"));
    gtk::style_context_add_provider_for_display(
        &display,
        &default_css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let user_css = CssProvider::new();
    user_css.load_from_path(CONFIG_DIR.join("style.css"));
    gtk::style_context_add_provider_for_display(
        &display,
        &user_css,
        gtk::STYLE_PROVIDER_PRIORITY_USER,
    );
}
