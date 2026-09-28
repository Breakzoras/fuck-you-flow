import type { DailyStat } from "./api";

// The typing speed the time-saved figure assumes. The backend passes the same 40.0 to
// `stats_summary` (src-tauri/src/commands.rs), which uses it for the all-time total.
export const TYPING_WPM = 40;

/** The local calendar day as YYYY-MM-DD, the same key the backend writes into stats_daily. */
export function localDayKey(d: Date): string {
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}

/**
 * Today's words and minutes saved, for the Home "today" row. Same rule as the all-time total
 * in the backend: minutes it would take to type the words, minus the minutes spent speaking,
 * never below zero. A day without dictations reads as zero, whatever happened yesterday.
 */
export function todayFacts(daily: DailyStat[], dayKey: string): { words: number; savedMinutes: number } {
  const d = daily.find((x) => x.day === dayKey);
  if (!d) return { words: 0, savedMinutes: 0 };
  return { words: d.words, savedMinutes: Math.max(0, d.words / TYPING_WPM - d.audio_ms / 60000) };
}
