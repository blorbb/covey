// These must match `covey_tauri::ipc::frontend::Event`.

import { Channel, invoke } from "@tauri-apps/api/core";

import type { Command, Event, Hotkey, ListItem, ListStyle } from "./bindings";
import * as keys from "./keys";
import { Settings } from "./settings.svelte";
import type { DeepReadonly } from "./utils";

export type CommandInfo = Command & { customHotkey?: Hotkey };

export class Menu {
  public items = $state<ListItem[]>([]);
  public style = $state<ListStyle | undefined>();
  public selection = $state<number>(0);
  public inputText = $state<string>("");
  // this is only updated by plugins, so no need to keep live
  // with the actual selection when changed by UI
  public textSelection = $state<[number, number]>([0, 0]);

  // definitely assigned in `new`.
  private settings!: Settings;

  private constructor() {}

  public static async new(): Promise<Menu> {
    console.debug("calling new");
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
        case "reload":
          self.items = [];
          self.selection = 0;
          // re-query the current input
          self.query();
          // reload settings
          void Settings.new().then((settings) => (self.settings = settings));
      }
    };

    await invoke("setup", { events });
    self.settings = await Settings.new();
    return self;
  }

  public query() {
    void invoke("query", { text: this.inputText });
  }

  public activateById(commandId: string) {
    void invoke("activate", {
      listItemId: this.items[this.selection].id,
      commandName: commandId,
    });
  }

  /**
   * Tries to activate a command from a keyboard event.
   *
   * Returns `true` if something was activated.
   */
  public activateByEvent(ev: KeyboardEvent): boolean {
    const pressedHotkey = keys.hotkeyFromKeyboardEvent(ev);
    if (pressedHotkey == null) return false;

    return this.activateByHotkey(pressedHotkey);
  }

  public activateByHotkey(pressedHotkey: Hotkey): boolean {
    const commands = this.getAvailableCommands();

    // find a command id that is in the `availableCommands` and matches
    // the hotkey (either custom or default)
    const commandId = commands.find((cmd) => {
      const hotkey = cmd.customHotkey ?? cmd["default-hotkey"];
      return hotkey != null && keys.hotkeysEqual(hotkey, pressedHotkey);
    })?.id;

    if (commandId == null) return false;

    // activate the found command
    this.activateById(commandId);
    return true;
  }

  public showSettingsWindow() {
    void invoke("show_settings_window");
  }

  public currentItem(): ListItem | undefined {
    return this.items[this.selection];
  }

  /// Returns an empty list if there is not currently selected item
  public getAvailableCommands(): DeepReadonly<CommandInfo[]> {
    // get the config of the currently focused plugin
    const currentItem = this.currentItem();
    if (currentItem == null) return [];
    const pluginId = currentItem.id.pluginId;
    const pluginConfig = this.settings.getPlugin(pluginId);
    const pluginManifest = this.settings.manifests[pluginId];

    return (
      pluginManifest.commands
        .filter((cmd) => currentItem.availableCommands.includes(cmd.id))
        // add custom hotkey to the info
        .map((cmd) => ({
          ...cmd,
          customHotkey: pluginConfig?.commands[cmd.id],
        }))
    );
  }
}
