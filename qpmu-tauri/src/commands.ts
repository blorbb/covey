import { invoke } from "@tauri-apps/api/core";

export const query = async (text: string) => {
  await invoke("query", { text });
};

export const activate = async (id: number) => {
  await invoke("activate", { listItemId: id });
};

export const altActivate = async (id: number) => {
  await invoke("alt_activate", { listItemId: id });
};

export const complete = async (id: number) => {
  await invoke("complete", { listItemId: id });
};
