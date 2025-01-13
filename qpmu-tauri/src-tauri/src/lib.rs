use tauri::{Emitter, Manager};

// use AppHandle instead of Window so that these commands only affect
// the main window (don't want these to affect the settings)
#[tauri::command]
fn set_window_size(app: tauri::AppHandle, width: f64, height: f64) {
    eprintln!("setting window size to {width}x{height}");
    if let Some(window) = app.get_webview_window("main") {
        window
            .set_size(tauri::Size::Logical(tauri::LogicalSize { width, height }))
            .unwrap();
    } else {
        eprintln!("WARN: main window was not found");
    }
}

#[tauri::command]
fn hide_window(app: tauri::AppHandle) {
    eprintln!("hiding window");
    if let Some(window) = app.get_webview_window("main") {
        window.hide().unwrap();
    } else {
        eprintln!("WARN: main window was not found");
    }
}

#[tauri::command]
fn show_window(app: tauri::AppHandle) {
    eprintln!("hiding window");
    if let Some(window) = app.get_webview_window("main") {
        window.show().unwrap();
        window.set_focus().unwrap();
    } else {
        eprintln!("WARN: main window was not found");
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let Some(main_window) = app.get_webview_window("main") else {
                panic!("missing main window")
            };

            main_window.on_window_event({
                let main_window = main_window.clone();
                move |ev| match ev {
                    tauri::WindowEvent::CloseRequested { api, .. } => {
                        api.prevent_close();
                        main_window.hide().unwrap();
                    }
                    tauri::WindowEvent::Focused(focused) => {
                        if *focused {
                            main_window.emit("focus-input", ()).unwrap();
                        } else {
                            main_window.hide().unwrap();
                        }
                    }
                    _ => {}
                }
            });

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            set_window_size,
            hide_window,
            show_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
