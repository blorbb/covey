// These must match `covey_tauri::ipc::frontend::Event`.

import { invoke } from "@tauri-apps/api/core";

import type { GlobalConfig, PluginConfig, PluginManifest } from "./bindings";

type OrderedGlobalConfig = {
  plugins: { name: string; pluginConfig: PluginConfig }[];
};

export class Settings {
  public globalConfig: OrderedGlobalConfig = $state() as OrderedGlobalConfig;

  private constructor(config: GlobalConfig) {
    this.globalConfig = {
      ...config,
      plugins: Object.entries(config.plugins)
        .map(([name, pluginConfig]) => ({
          name,
          pluginConfig,
        })),
    };
  }

  public static async new(): Promise<Settings> {
    const config = await invoke<GlobalConfig>("get_global_config");
    const self = new Settings(config);
    return self;
  }

  public updateBackendConfig(): void {
    void invoke("set_global_config", {
      config: {
        ...this.globalConfig,
        plugins: Object.fromEntries(
          this.globalConfig.plugins.map(({ name, pluginConfig }) => [
            name,
            pluginConfig,
          ]),
        ),
      } satisfies GlobalConfig,
    });
  }

  public async manifestOf(pluginName: string): Promise<PluginManifest> {
    return invoke("get_manifest", { pluginName });
  }
}
