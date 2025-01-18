// These must match `qpmu_tauri::ipc::frontend::Event`.

import { Channel, invoke } from "@tauri-apps/api/core";
import * as commands from "./commands";
import type { ListItem } from "./bindings/ListItem";
import type { ListStyle } from "./bindings/ListStyle";
import type { Event } from "./bindings/Event";

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
        default:
          unreachable(msg);
      }
    };

    await invoke("setup", { events });
    return self;
  }

  public activate() {
    void commands.activate(this.items[this.selection].id);
  }

  public altActivate() {
    void commands.altActivate(this.items[this.selection].id);
  }

  public complete() {
    void commands.complete(this.items[this.selection].id);
  }
}

function unreachable(x: never): never {
  return x;
}
