import { invoke } from "@tauri-apps/api/core";

export const query = async (text: string) => {
  await invoke("query", { text });
};

