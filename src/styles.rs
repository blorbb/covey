use relm4::gtk::{self, gdk::Display, CssProvider};

pub fn load_css() {
    let css = CssProvider::new();
    css.load_from_data(include_str!("../styles/style.css"));
    gtk::style_context_add_provider_for_display(
        &Display::default().expect("could not connect to a display"),
        &css,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
