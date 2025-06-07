import { invoke } from "@tauri-apps/api/core";

export async function showSettingsWindow(): Promise<void> {
  await invoke("show_settings_window");
}

export async function showMenuWindow(): Promise<void> {
  await invoke("show_menu_window");
}

export async function hideMenuWindow(): Promise<void> {
  await invoke("hide_menu_window");
}
