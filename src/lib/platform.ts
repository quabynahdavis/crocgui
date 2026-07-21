import { invoke } from "@tauri-apps/api/core";

let _crocAvailable: boolean | null = null;
let _lastCheck = 0;
const CACHE_TTL_MS = 30_000;

export async function isCrocAvailable(): Promise<boolean> {
  const now = Date.now();
  if (_crocAvailable !== null && (now - _lastCheck) < CACHE_TTL_MS) {
    return _crocAvailable;
  }
  try {
    _crocAvailable = await invoke<boolean>("check_croc_available");
  } catch {
    _crocAvailable = false;
  }
  _lastCheck = now;
  return _crocAvailable;
}
