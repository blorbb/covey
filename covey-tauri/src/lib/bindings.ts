/**
 * Re-exported bindings with some overridden to make records required.
 */

import type { Action } from "./bindings/Action";
import type { Command } from "./bindings/Command";
import type { CommandId } from "./bindings/CommandId";
import type { ConfigBool } from "./bindings/ConfigBool";
import type { ConfigFilePath } from "./bindings/ConfigFilePath";
import type { ConfigFolderPath } from "./bindings/ConfigFolderPath";
import type { ConfigInt } from "./bindings/ConfigInt";
import type { ConfigList } from "./bindings/ConfigList";
import type { ConfigMap } from "./bindings/ConfigMap";
import type { ConfigSchema } from "./bindings/ConfigSchema";
import type { ConfigStr } from "./bindings/ConfigStr";
import type { ConfigStruct as ConfigStructBinding } from "./bindings/ConfigStruct";
import type { ConfigType } from "./bindings/ConfigType";
import type { Event } from "./bindings/Event";
import type { GlobalConfig as GlobalConfigBinding } from "./bindings/GlobalConfig";
import type { Hotkey } from "./bindings/Hotkey";
import type { Icon } from "./bindings/Icon";
import type { Key } from "./bindings/Key";
import type { ListItem } from "./bindings/ListItem";
import type { ListItemId } from "./bindings/ListItemId";
import type { ListStyle } from "./bindings/ListStyle";
import type { PluginConfig as PluginConfigBinding } from "./bindings/PluginConfig";
import type { PluginManifest as PluginManifestBinding } from "./bindings/PluginManifest";
import type { JsonValue as JsonValueBinding } from "./bindings/serde_json/JsonValue";

export type {
  Action,
  Command,
  CommandId,
  ConfigBool,
  ConfigFilePath,
  ConfigFolderPath,
  ConfigInt,
  ConfigList,
  ConfigMap,
  ConfigSchema,
  ConfigStr,
  ConfigStruct,
  ConfigType,
  Event,
  GlobalConfig,
  Hotkey,
  Icon,
  JsonValue,
  Key,
  ListItem,
  ListItemId,
  ListStyle,
  PluginConfig,
  PluginManifest,
};

type JsonValue =
  | Exclude<JsonValueBinding, { [x: string]: JsonValueBinding | undefined }>
  | { [x: string]: JsonValue };

type ConfigStruct = ConfigStructBinding & {
  fields: Record<string, ConfigType>;
};

type GlobalConfig = GlobalConfigBinding & {
  plugins: Record<string, PluginConfig>;
};

type PluginConfig = PluginConfigBinding & {
  config: Record<string, JsonValue>;
  commands: Record<string, Hotkey>;
};

type PluginManifest = PluginManifestBinding & {
  schema: Record<string, ConfigSchema>;
  commands: Record<CommandId, Command>;
};
