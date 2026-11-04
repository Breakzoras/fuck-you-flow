// What the badge says about time while somebody talks.
//
// Under two minutes nothing changes from before: tenths of a second, which
// is what a quick dictation wants to see. From two minutes on the tenths only
// flicker, so the clock turns into minutes and seconds (Lu, 4 November 2026).

const CLOCK_FROM_SECONDS = 120;
/// How long before the end of a part the badge says what is about to happen.
export const LIMIT_WARNING_SECONDS = 60;

export function recordingClock(seconds: number): string {
  if (!Number.isFinite(seconds) || seconds < 0) return "0.0s";
  if (seconds < CLOCK_FROM_SECONDS) return `${seconds.toFixed(1)}s`;
  const whole = Math.floor(seconds);
  const h = Math.floor(whole / 3600);
  const m = Math.floor((whole % 3600) / 60);
  const s = String(whole % 60).padStart(2, "0");
  return h > 0 ? `${h}:${String(m).padStart(2, "0")}:${s}` : `${m}:${s}`;
}

/// The warning shortly before a part ends, as a translation key, or null.
/// Hands-free writes what was said and keeps listening; a held key stops.
export function limitWarningKey(left: number | undefined, handsFree: boolean): string | null {
  if (left === undefined || !Number.isFinite(left)) return null;
  if (left > LIMIT_WARNING_SECONDS) return null;
  return handsFree ? "msg_limit_soon_continue" : "msg_limit_soon_stop";
}

/// The "still writing" line after the key is released, as a key and its count.
export function piecesLeftKey(left: number): { key: string; vars: { n: number } } | null {
  if (!(left > 0)) return null;
  return { key: left === 1 ? "msg_pieces_left_one" : "msg_pieces_left", vars: { n: left } };
}
