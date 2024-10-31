use gio::{prelude::*, spawn_blocking};
use gtk::gdk::Key;
use gtk::glib::clone;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Entry, Label, ListBox, ListBoxRow, Orientation};
use std::fs;
use std::io::Write;
use std::net::{TcpListener, TcpStream};

const WINDOW_WIDTH: i32 = 800;
const SOCKET_ADDR: &str = "127.0.0.1:7547";

fn main() {
    match TcpListener::bind(SOCKET_ADDR) {
        Ok(listener) => {
            let app = Application::new(Some("com.blorbb.qlist"), Default::default());
            app.connect_activate(move |app| build_ui(app, listener.try_clone().unwrap()));
            app.run();
        }
        Err(_) => {
            // another instance running
            println!("activating other instance");
            match TcpStream::connect(SOCKET_ADDR) {
                Ok(mut stream) => stream
                    .write_all(b"1")
                    .unwrap_or_else(|e| eprintln!("error writing to stream: {e}")),
                Err(e) => eprintln!("error connecting to stream: {e}"),
            }
        }
    }
}

fn build_ui(app: &Application, listener: TcpListener) {
    // Create the main application window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("qlist")
        .decorated(false)
        .hide_on_close(true)
        .deletable(false)
        .can_focus(true)
        .vexpand(true)
        .resizable(true)
        .default_width(WINDOW_WIDTH)
        .build();

    // Main vertical layout
    let vbox = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .build();

    // Input entry field
    let entry = Entry::builder().placeholder_text("Search...").build();
    vbox.append(&entry);

    // Results list
    let list_box = ListBox::new();
    list_box.set_selection_mode(gtk::SelectionMode::Browse);
    list_box.connect_row_activated(clone!(
        #[weak]
        entry,
        move |_, _| _ = entry.grab_focus_without_selecting()
    ));

    vbox.append(&list_box);

    // Connect to the entry's "changed" signal to update search results
    entry.connect_changed(clone!(
        #[weak]
        list_box,
        #[weak]
        window,
        move |entry| {
            // Clear the current list
            list_box.remove_all();

            // Get the current text from the entry
            let query = entry.text().to_string();

            // Filter applications based on the query
            let apps = find_applications(&query);
            for app in apps {
                let row = ListBoxRow::new();
                let label = Label::new(Some(&app));
                row.set_child(Some(&label));
                list_box.append(&row);
            }
            list_box.select_row(list_box.row_at_index(0).as_ref());
            window.set_default_size(WINDOW_WIDTH, -1);
        }
    ));
    list_box.selected_row().map(|a| a.next_accessible_sibling());

    let global_events = gtk::EventControllerKey::new();
    global_events.connect_key_pressed(clone!(
        #[weak]
        window,
        #[weak]
        list_box,
        #[weak]
        entry,
        #[upgrade_or]
        glib::Propagation::Proceed,
        move |_self, key, _keycode, _modifiers| {
            match key {
                Key::Escape => window.close(),
                Key::Up => {
                    list_box.select_row(
                        list_box
                            .selected_row()
                            .and_then(|a| a.prev_sibling())
                            .and_then(|a| a.downcast::<ListBoxRow>().ok())
                            .as_ref(),
                    );
                }
                Key::Down => {
                    list_box.select_row(
                        list_box
                            .selected_row()
                            .and_then(|a| a.next_sibling())
                            .and_then(|a| a.downcast::<ListBoxRow>().ok())
                            .as_ref(),
                    );
                }
                _ => return glib::Propagation::Proceed,
            }
            entry.grab_focus_without_selecting();
            glib::Propagation::Stop
        }
    ));

    let focus_events = gtk::EventControllerFocus::new();
    focus_events.connect_leave(clone!(
        #[weak]
        window,
        move |_| {
            window.close();
        }
    ));

    window.add_controller(global_events);
    window.add_controller(focus_events);
    window.set_child(Some(&vbox));
    window.set_default_widget(Some(&entry));
    window.present();

    let (tx, rx) = async_channel::bounded(1);
    spawn_blocking(move || {
        for stream in listener.incoming() {
            eprintln!("got stream {stream:?}");
            if stream.is_ok() {
                tx.send_blocking(())
                    .unwrap_or_else(|e| eprintln!("failed to send stream: {e}"));
            }
        }
    });

    glib::spawn_future_local(async move {
        while rx.recv().await.is_ok() {
            window.present();
        }
    });
}

fn find_applications(query: &str) -> Vec<String> {
    let mut results = Vec::new();

    if let Ok(entries) = fs::read_dir("/usr/share/applications") {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                if name.contains(query) {
                    results.push(name.to_string());
                }
            }
        }
    }

    results
}
