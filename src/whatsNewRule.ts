/** Kept equal to WHATS_NEW_ID in src-tauri/src/settings.rs. */
export const WHATS_NEW_ID = "app-mic";

export interface WhatsNewState {
  /** Dictating through Claude's and Codex's microphones exists on Windows only. */
  windows: boolean;
  firstRunDone: boolean;
  /** general.whats_new_seen: "" after an update, the id with "~shown" once
   *  the app brought the window forward, the bare id once dealt with. */
  seen: string;
}

/** The "What's new" card belongs on screen. */
export function wantsWhatsNew(s: WhatsNewState): boolean {
  return s.windows && s.firstRunDone && s.seen !== WHATS_NEW_ID;
}

/** The app may bring its window forward for the card: once, the first time. */
export function mayBringFront(s: WhatsNewState): boolean {
  return wantsWhatsNew(s) && s.seen === "";
}
