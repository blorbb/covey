// These must match `qpmu_tauri::ipc::frontend::Event`.

import { Channel, invoke } from "@tauri-apps/api/core";

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

  constructor() {
    console.log("!!");
    const events = new Channel<Event>();
    events.onmessage = (msg) => {
      switch (msg.kind) {
        case "setInput":
          this.inputText = msg.contents;
          this.textSelection = msg.selection;
          break;
        case "setList":
          this.items = msg.items;
          this.style = msg.style;
          this.selection = 0;
          break;
        default:
          unreachable(msg);
      }
    };
    void invoke("setup", { events });
  }
}

function unreachable(x: never): never {
  return x;
}
