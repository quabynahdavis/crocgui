import { invoke } from "@tauri-apps/api/core";

let _crocAvailable: boolean | null = null;

export async function isCrocAvailable(): Promise<boolean> {
  if (_crocAvailable === null) {
    try {
      _crocAvailable = await invoke<boolean>("check_croc_available");
    } catch {
      _crocAvailable = false;
    }
  }
  return _crocAvailable;
}
