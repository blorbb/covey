import { invoke } from "@tauri-apps/api/core";
import type { Hotkey } from "./bindings/Hotkey";
import type { ListItemId } from "./bindings/ListItemId";

export const query = async (text: string) => {
  await invoke("query", { text });
};

export const activate = async (id: ListItemId) => {
  await invoke("activate", { listItemId: id });
};

export const altActivate = async (id: ListItemId) => {
  await invoke("alt_activate", { listItemId: id });
};

export const hotkeyActivate = async (id: ListItemId, hotkey: Hotkey) => {
  await invoke("alt_activate", { listItemId: id, hotkey });
};

export const complete = async (id: ListItemId) => {
  await invoke("complete", { listItemId: id });
};
