/** Kept equal to WHATS_NEW_ID in src-tauri/src/settings.rs. Was "app-mic"
 *  (0.9.19, Windows only); "disk-clean" (0.9.23) applies everywhere. */
export const WHATS_NEW_ID = "disk-clean";

export interface WhatsNewState {
  firstRunDone: boolean;
  /** general.whats_new_seen: "" after an update from before 0.9.19, an older
   *  card's id after a later update, this id with "~shown" once the app
   *  brought the window forward, the bare id once dealt with. */
  seen: string;
}

/** The "What's new" card belongs on screen. */
export function wantsWhatsNew(s: WhatsNewState): boolean {
  return s.firstRunDone && s.seen !== WHATS_NEW_ID;
}

/** The app may bring its window forward for the card: once, the first time.
 *  Until 0.9.23 this asked for an empty field, which only a pre-0.9.19 file
 *  has; anyone who had dealt with the previous card would never have seen
 *  the window come forward for the next one. */
export function mayBringFront(s: WhatsNewState): boolean {
  return wantsWhatsNew(s) && s.seen !== `${WHATS_NEW_ID}~shown`;
}
