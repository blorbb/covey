// These must match `covey_tauri::ipc::frontend::Event`.

import { Channel, invoke } from "@tauri-apps/api/core";

import type { Event } from "./bindings/Event";
import type { Hotkey } from "./bindings/Hotkey";
import type { Key } from "./bindings/Key";
import type { ListItem } from "./bindings/ListItem";
import type { ListStyle } from "./bindings/ListStyle";
import * as commands from "./commands";

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

    const key = keyNameToKey(ev.key);
    if (key === undefined) return;

    const hotkey: Hotkey = {
      key,
      ctrl: ev.ctrlKey,
      alt: ev.altKey,
      shift: ev.shiftKey,
      meta: ev.metaKey,
    };
    void commands.hotkeyActivate(this.items[this.selection].id, hotkey);
  }
}

const keyNameToKey = (keyCased: string): Key | undefined => {
  const key = keyCased.toLowerCase();

  // Alphabetical
  if (/^[a-z]$/.test(key)) {
    return key as Key;
  }

  // Numerical
  if (/^[0-9]$/.test(key)) {
    return ("digit" + key) as Key;
  }

  // make sure this starts from shift+0, shift+1, ... shift+9
  // not shift+1, ..., shift+0
  const digitShiftIndex = ")!@#$%^&*(".indexOf(key);
  if (digitShiftIndex !== -1) {
    return ("digit" + digitShiftIndex.toString()) as Key;
  }

  // f* keys
  if (key.startsWith("f")) {
    const fNum = Number.parseInt(key.slice(1), 10);
    if (1 <= fNum && fNum <= 24) {
      return key as Key;
    }
  }

  switch (key) {
    case "`":
    case "~":
      return "backtick";
    case "-":
    case "_":
      return "hyphen";
    case "=":
    case "+":
      return "equal";
    case "tab":
      return "tab";
    case "[":
    case "{":
      return "leftBracket";
    case "]":
    case "}":
      return "rightBracket";
    case "\\":
    case "|":
      return "backslash";
    case ";":
    case ":":
      return "semicolon";
    case "'":
    case '"':
      return "apostrophe";
    case "enter":
      return "enter";
    case ",":
    case "<":
      return "comma";
    case ".":
    case ">":
      return "period";
    case "/":
    case "?":
      return "slash";
    default:
      return;
  }
};
