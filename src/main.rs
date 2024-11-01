use clap::{Parser, Subcommand};
use color_eyre::eyre::{Context, ContextCompat, Result};
use gio::prelude::*;
use gtk::gdk::Key;
use gtk::glib::clone;
use gtk::{prelude::*, ScrolledWindow, Window};
use gtk::{Application, ApplicationWindow, Entry, ListBoxRow, Orientation};
use install::install_plugin;
use plugins::{initialise_plugin, ListItem, Plugin, PluginAction};
use result_list::ResultList;
use std::cell::RefCell;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::process::Stdio;
use std::rc::Rc;
use std::sync::LazyLock;
use std::time::Duration;
use std::{fs, process};
use tokio::runtime::Runtime;

const WINDOW_WIDTH: i32 = 800;
const MAX_LIST_HEIGHT: i32 = 600;
const SOCKET_ADDR: &str = "127.0.0.1:7547";

static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| Runtime::new().expect("runtime must succeed"));

mod install;
mod plugins;
mod result_list;
mod styles;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    Install { rest: Vec<String> },
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    match args.command {
        Some(Command::Install { rest }) => {
            process::exit(install_plugin(&rest)?.code().unwrap_or(1));
        }
        None => try_run_instance(),
    }
}

fn try_run_instance() -> Result<()> {
    match TcpListener::bind(SOCKET_ADDR) {
        Ok(listener) => {
            let exit_code = new_instance(listener)?;
            process::exit(exit_code.value());
        }
        Err(_) => {
            // another instance running
            println!("activating other instance");
            TcpStream::connect(SOCKET_ADDR)?
                .write_all(b"1")
                .context("error writing to stream")?;
        }
    }

    Ok(())
}

fn new_instance(listener: TcpListener) -> Result<glib::ExitCode> {
    let cfg = dirs::config_dir().context("could not open config directory")?;
    let plugins = cfg.join("qpmu").join("plugins");
    if !plugins.is_dir() {
        fs::create_dir_all(&plugins).context("could not create qpmu/plugins directory")?;
    }

    let plugins: Vec<_> = plugins
        .read_dir()?
        .filter_map(|path| Some(path.ok()?.path()))
        .filter(|p| p.extension().is_some_and(|s| s.to_str() == Some("wasm")))
        .inspect(|p| {
            println!(
                "loading plugin {}",
                p.file_stem()
                    .expect("path should be a file")
                    .to_str()
                    .expect("plugin should have utf-8 file name")
            )
        })
        .filter_map(|path| {
            initialise_plugin(&path)
                .inspect_err(|e| {
                    eprintln!(
                        "failed to load plugin {path}: {e}",
                        path = path.to_string_lossy()
                    )
                })
                .ok()
        })
        .map(|plugin| Rc::new(RefCell::new(plugin)))
        .collect();

    let app = Application::new(Some("com.blorbb.qpmu"), Default::default());
    app.connect_startup(|_| styles::load_css().unwrap());
    app.connect_activate(move |app| build_ui(app, listener.try_clone().unwrap(), plugins.clone()));
    Ok(app.run())
}

fn build_ui(app: &Application, listener: TcpListener, plugins: Vec<Rc<RefCell<Plugin>>>) {
    // Create the main application window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("qpmu")
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

    let entry = Entry::builder().placeholder_text("Search...").build();
    entry.add_css_class("main-entry");
    vbox.append(&entry);

    let scroller = ScrolledWindow::builder()
        .min_content_height(0)
        .max_content_height(MAX_LIST_HEIGHT)
        .propagate_natural_height(true)
        .build();
    vbox.append(&scroller);

    let results = ResultList::new();
    scroller.set_child(Some(results.list_box()));

    let active_plugin = Rc::new(RefCell::new(None::<Rc<RefCell<Plugin>>>));

    results.connect_row_activated(clone!(
        #[weak]
        entry,
        #[weak]
        window,
        #[strong]
        active_plugin,
        move |_, _, _, selected| {
            entry.grab_focus_without_selecting();
            activate_row(&selected, &active_plugin, window.into());
        }
    ));

    // Connect to the entry's "changed" signal to update search results
    entry.connect_changed(clone!(
        #[strong]
        results,
        #[weak]
        window,
        #[strong]
        active_plugin,
        #[strong]
        plugins,
        move |entry| {
            // Clear the current list
            results.clear();

            // Get the current text from the entry
            let query = entry.text().to_string();

            *active_plugin.borrow_mut() = plugins.first().map(Rc::clone);
            let result = active_plugin
                .borrow()
                .as_ref()
                .map(|p| p.borrow_mut().call_input(&query).unwrap())
                .unwrap_or_default();

            // Filter applications based on the query
            // let apps = find_applications(&query);
            scroller.set_visible(!result.is_empty());
            for app in result {
                results.push(app);
            }
            results
                .list_box()
                .select_row(results.list_box().row_at_index(0).as_ref());
            window.set_default_size(WINDOW_WIDTH, -1);
        }
    ));

    let global_events = gtk::EventControllerKey::new();
    global_events.connect_key_pressed(clone!(
        #[weak]
        window,
        #[strong]
        results,
        #[weak]
        entry,
        #[weak]
        active_plugin,
        #[upgrade_or]
        glib::Propagation::Proceed,
        move |_self, key, _keycode, _modifiers| {
            match key {
                Key::Escape => window.close(),
                Key::Up => {
                    results.list_box().select_row(
                        results
                            .list_box()
                            .selected_row()
                            .and_then(|a| a.prev_sibling())
                            .and_then(|a| a.downcast::<ListBoxRow>().ok())
                            .as_ref(),
                    );
                }
                Key::Down => {
                    results.list_box().select_row(
                        results
                            .list_box()
                            .selected_row()
                            .and_then(|a| a.next_sibling())
                            .and_then(|a| a.downcast::<ListBoxRow>().ok())
                            .as_ref(),
                    );
                }
                Key::Return => {
                    dbg!("return");
                    if let Some(row) = results.list_box().selected_row() {
                        dbg!(&row);
                        activate_row(
                            &results.get_item(row.index() as usize).unwrap(),
                            &active_plugin,
                            window.into(),
                        );
                    }
                }
                _ => return glib::Propagation::Proceed,
            }
            entry.grab_focus_without_selecting();
            glib::Propagation::Stop
        }
    ));
    global_events.set_propagation_phase(gtk::PropagationPhase::Capture);

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

    RUNTIME.spawn(async move {
        listener.set_nonblocking(true).unwrap();
        let listener = tokio::net::TcpListener::from_std(listener).unwrap();
        loop {
            if let Ok(stream) = listener.accept().await {
                eprintln!("got stream {stream:?}");

                tx.send(())
                    .await
                    .unwrap_or_else(|e| eprintln!("failed to send stream: {e}"));
            }
        }
    });

    glib::spawn_future_local(async move {
        while rx.recv().await.is_ok() {
            window.present();
            entry.grab_focus();
            // need a short delay for the select_region to actually work. no clue why.
            glib::timeout_add_local(
                Duration::from_millis(10),
                clone!(
                    #[weak]
                    entry,
                    #[upgrade_or]
                    glib::ControlFlow::Break,
                    move || {
                        entry.select_region(0, dbg!(entry.text_length()) as i32);
                        glib::ControlFlow::Break
                    }
                ),
            );
        }
    });
}

fn activate_row(
    item: &ListItem,
    // TODO: fix this cursed type
    active_plugin: &RefCell<Option<Rc<RefCell<Plugin>>>>,
    window: Window,
) {
    let actions = active_plugin
        .borrow()
        .as_ref()
        .unwrap()
        .borrow_mut()
        .call_activate(item)
        .unwrap();
    execute_actions(actions, window).unwrap();
}

fn execute_actions(actions: Vec<PluginAction>, window: Window) -> Result<()> {
    for action in actions {
        match action {
            PluginAction::Close => window.close(),
            PluginAction::RunCommand((cmd, args)) => {
                std::process::Command::new(cmd).args(args).spawn()?;
            }
            PluginAction::RunCommandString(cmd) => {
                std::process::Command::new("sh")
                    .arg("-c")
                    .arg(cmd)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()?;
            }
        }
    }

    Ok(())
}
