import { invoke } from "@tauri-apps/api/core";

export const query = async (text: string) => {
  await invoke("query", { text });
};

export const activate = async (id: bigint) => {
  await invoke("activate", { listItemId: id });
};

export const altActivate = async (id: bigint) => {
  await invoke("alt_activate", { listItemId: id });
};

export const complete = async (id: bigint) => {
  await invoke("complete", { listItemId: id });
};
