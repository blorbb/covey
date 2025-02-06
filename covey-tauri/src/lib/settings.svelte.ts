import { invoke } from "@tauri-apps/api/core";

import type {
  GlobalConfig,
  Key,
  PluginConfig,
  PluginManifest,
} from "./bindings";
import type { DeepReadonly } from "./utils";

export class Settings {
  // definitely assigned in constructor so will not be undefined
  public globalConfig = $state() as GlobalConfig;
  public manifests = $state() as DeepReadonly<Record<string, PluginManifest>>;

  private constructor(
    config: GlobalConfig,
    manifests: DeepReadonly<Record<string, PluginManifest>>,
  ) {
    this.globalConfig = config;
    this.manifests = manifests;
  }

  public static async new(): Promise<Settings> {
    const config = await invoke<GlobalConfig>("get_global_config");
    console.debug("received settings", config);

    const manifests = await Promise.all(
      config.plugins.map<Promise<[Key, PluginManifest]>>(async (plugin) => [
        plugin.id,
        await invoke("get_manifest", {
          pluginName: plugin.id,
        }),
      ]),
    );

    console.debug("received manifests", manifests);

    const self = new Settings(config, Object.fromEntries(manifests));
    return self;
  }

  public updateBackendConfig(): void {
    console.debug("updating config to new");
    void invoke("set_global_config", {
      config: this.globalConfig,
    });
  }

  public getPlugin(pluginId: string): PluginConfig | undefined {
    return this.globalConfig.plugins.find((plugin) => plugin.id === pluginId);
  }
}
