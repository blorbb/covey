import { invoke } from "@tauri-apps/api/core";

export type Src = string;

export class IconCache {
  private readonly cache: Map<string, Promise<Src>> = new Map();

  /**
   * Finds the icon from a given file path, reading the filesystem if needed.
   */
  public async open(filePath: string): Promise<Src> {
    const cached = this.cache.get(filePath);
    if (cached) {
      return await cached;
    } else {
      // fill with a promise immediately to avoid multiple fetches
      const prom = invoke<Uint8Array>("read_any_file", { path: filePath }).then(
        (bytes) => {
          // svgs need an explicit MIME type to be rendered correctly.
          let blob: Blob;
          if (filePath.endsWith("svg")) {
            blob = new Blob([bytes], { type: "image/svg+xml" });
          } else {
            blob = new Blob([bytes]);
          }
          return URL.createObjectURL(blob);
        },
      );

      this.cache.set(filePath, prom);

      return await prom;
    }
  }
}
