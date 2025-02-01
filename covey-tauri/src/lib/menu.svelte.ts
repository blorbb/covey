// These must match `covey_tauri::ipc::frontend::Event`.

import { Channel, invoke } from "@tauri-apps/api/core";

import type { Event, Hotkey, ListItem, ListStyle } from "./bindings";
import * as keys from "./keys";

export class Menu {
  public items = $state<ListItem[]>([]);
  public style = $state<ListStyle | undefined>();
  public selection = $state<number>(0);
  public inputText = $state<string>("");
  // this is only updated by plugins, so no need to keep live
  // with the actual selection when changed by UI
  public textSelection = $state<[number, number]>([0, 0]);

  private constructor() {}

  public static async new(): Promise<Menu> {
    const self = new Menu();

    const events = new Channel<Event>();
    events.onmessage = (msg) => {
      switch (msg.kind) {
        case "setInput":
          self.inputText = msg.contents;
          self.textSelection = msg.selection;
          break;
        case "setList":
          self.items = msg.items;
          self.style = msg.style ?? undefined;
          self.selection = 0;
          break;
      }
    };

    await invoke("setup", { events });
    return self;
  }

  public query() {
    void invoke("query", { text: this.inputText });
  }

  public activate(name: string) {
    void invoke("activate", {
      listItemId: this.items[this.selection].id,
      commandName: name,
    });
  }

  // TODO: retrieve command settings from rust side
  // make left click = enter.
  public maybeHotkeyActivate(ev: KeyboardEvent) {
    // require one of ctrl/alt/meta to be pressed to be considered a hotkey
    if (!(ev.ctrlKey || ev.altKey || ev.metaKey)) return;

    const key = keys.symbolToKeyCode(ev.key);
    if (key === undefined) return;

    const hotkey: Hotkey = {
      key,
      ctrl: ev.ctrlKey,
      alt: ev.altKey,
      shift: ev.shiftKey,
      meta: ev.metaKey,
    };
    // this.activate("activate");
  }

  public showSettingsWindow() {
    void invoke("show_settings_window");
  }
}
