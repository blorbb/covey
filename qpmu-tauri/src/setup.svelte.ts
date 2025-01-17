// These must match `qpmu_tauri::ipc::frontend::Event`.

import { Channel, invoke } from "@tauri-apps/api/core";
import * as commands from "./commands";

export type Event = Readonly<
  | { kind: "setInput"; contents: string; selection: [number, number] }
  | { kind: "setList"; items: ListItem[]; style?: ListStyle }
>;

export type ListItem = Readonly<{
  title: string;
  description: string;
  id: number;
}>;

export type ListStyle = Readonly<
  { kind: "rows" | "grid" } | { kind: "gridWithColumns"; columns: number }
>;

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
          self.style = msg.style;
          self.selection = 0;
          break;
        default:
          unreachable(msg);
      }
    };

    await invoke("setup", { events });
    return self;
  }

  public activate() {
    commands.activate(this.items[this.selection].id);
  }

  public altActivate() {
    commands.altActivate(this.items[this.selection].id);
  }

  public complete() {
    commands.complete(this.items[this.selection].id);
  }
}

function unreachable(x: never): never {
  return x;
}
