import { invoke } from "@tauri-apps/api/core";

export interface CrocSettings {
  relay: string | null;
  curve: string | null;
  disableCompression: boolean;
  outputDir?: string | null;
}

export async function loadSettings(): Promise<CrocSettings> {
  try {
    const s: any = await invoke("get_settings");
    return {
      relay: s.relay || null,
      curve: s.curve || null,
      disableCompression: s.disable_compression || false,
      outputDir: s.output_dir || null,
    };
  } catch {
    return { relay: null, curve: null, disableCompression: false, outputDir: null };
  }
}
