/**
 * Re-exported bindings with some overridden to make records required.
 */

import type { Command } from "./bindings/Command";
import type { Event } from "./bindings/Event";
import type { Hotkey } from "./bindings/Hotkey";
import type { Icon } from "./bindings/Icon";
import type { Key } from "./bindings/Key";
import type { KeyCode } from "./bindings/KeyCode";
import type { KeyedList } from "./bindings/KeyedList";
import type { ListItem } from "./bindings/ListItem";
import type { ListItemId } from "./bindings/ListItemId";
import type { ListStyle } from "./bindings/ListStyle";
import type { PluginConfig as PluginConfigBinding } from "./bindings/PluginConfig";
import type { PluginConfigSchema as PluginConfigSchemaBinding } from "./bindings/PluginConfigSchema";
import type { PluginManifest } from "./bindings/PluginManifest";
import type { SchemaBool } from "./bindings/SchemaBool";
import type { SchemaFilePath } from "./bindings/SchemaFilePath";
import type { SchemaFolderPath } from "./bindings/SchemaFolderPath";
import type { SchemaInt } from "./bindings/SchemaInt";
import type { SchemaList as SchemaListBinding } from "./bindings/SchemaList";
import type { SchemaMap as SchemaMapBinding } from "./bindings/SchemaMap";
import type { SchemaStruct as SchemaStructBinding } from "./bindings/SchemaStruct";
import type { SchemaText } from "./bindings/SchemaText";
import type { SchemaType as SchemaTypeBinding } from "./bindings/SchemaType";
import type { JsonValue as JsonValueBinding } from "./bindings/serde_json/JsonValue";

export type {
  Command,
  Event,
  GlobalConfig,
  Hotkey,
  Icon,
  JsonValue,
  Key,
  KeyCode,
  KeyedList,
  ListItem,
  ListItemId,
  ListStyle,
  PluginConfig,
  PluginConfigSchema,
  PluginManifest,
  SchemaBool,
  SchemaFilePath,
  SchemaFolderPath,
  SchemaInt,
  SchemaList,
  SchemaMap,
  SchemaStruct,
  SchemaText,
  SchemaType,
};

type JsonValue =
  | Exclude<
      JsonValueBinding,
      { [x: string]: JsonValueBinding | undefined } | JsonValueBinding[]
    >
  | { [x: string]: JsonValue }
  | JsonValue[];

type SchemaStruct = SchemaStructBinding & {
  fields: Record<string, SchemaType>;
};

type SchemaMap = SchemaMapBinding & { "value-type": SchemaType };
type SchemaList = SchemaListBinding & { "item-type": SchemaType };

type SchemaType =
  | Exclude<
      SchemaTypeBinding,
      { struct: unknown } | { list: unknown } | { map: unknown }
    >
  | { struct: SchemaStruct }
  | { list: SchemaList }
  | { map: SchemaMap };

type PluginConfigSchema = PluginConfigSchemaBinding & {
  type: SchemaType;
};

type GlobalConfig = {
  plugins: KeyedList<PluginConfig>;
};

type PluginConfig = PluginConfigBinding & {
  config: Record<string, JsonValue>;
  commands: Record<Key, Hotkey>;
};
