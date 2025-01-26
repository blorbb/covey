import { invoke } from "@tauri-apps/api/core";

import type { GlobalConfig, PluginConfig, PluginManifest } from "./bindings";

type PluginList = { name: string; config: PluginConfig }[];

export class Settings {
  // definitely assigned in constructor so will not be undefined
  public globalConfig: GlobalConfig = $state() as GlobalConfig;

  private constructor(config: GlobalConfig) {
    this.globalConfig = config;
  }

  public static async new(): Promise<Settings> {
    const config = await invoke<GlobalConfig>("get_global_config");
    const self = new Settings(config);
    console.log("got config", config);
    return self;
  }

  public updateBackendConfig(): void {
    void invoke("set_global_config", {
      config: this.globalConfig,
    });
  }

  public get plugins() {
    return Object.entries(this.globalConfig.plugins).map(([name, config]) => ({
      name,
      config,
    }));
  }

  public set plugins(plugins: PluginList) {
    this.globalConfig.plugins = Object.fromEntries(
      plugins.map(({ name, config }) => [name, config]),
    );
  }

  public async manifestOf(pluginName: string): Promise<PluginManifest> {
    return invoke("get_manifest", { pluginName });
  }
}
