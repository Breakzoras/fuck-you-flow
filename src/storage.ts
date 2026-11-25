import { useCallback, useEffect, useState } from "react";
import { api, CleanResult, StorageReport } from "./api";
import { useApp } from "./hooks";

/** Fired after anything deletes files, so the gauge, Settings and the
 *  "What's new" card all show the same number at once. */
export const STORAGE_CHANGED = "fyf-storage-changed";

/** Sizing the folders reads every file name under them, so the rail asks
 *  once a minute rather than every few seconds like the card memory. */
const POLL_MS = 60_000;

/** Below this the rail shows "nothing to clean": a few kilobytes of
 *  leftovers are not worth a button. */
export const WORTH_CLEANING_BYTES = 1_000_000;

/** "612 MB", "1.4 GB", "820 KB". Decimal comma in Greek. Units of 1000, the
 *  same as `fmtBytes` beside the model list, so both show one number. */
export function formatBytes(bytes: number, lang: string): string {
  const units = ["B", "KB", "MB", "GB"];
  let v = bytes;
  let i = 0;
  while (v >= 1000 && i < units.length - 1) {
    v /= 1000;
    i++;
  }
  const digits = i >= 3 && v < 10 ? 1 : 0;
  const n = v.toLocaleString(lang === "el" ? "el-GR" : "en-US", { maximumFractionDigits: digits, minimumFractionDigits: digits });
  return `${n} ${units[i]}`;
}

export function useStorageReport(poll = true) {
  const { settings } = useApp();
  const [report, setReport] = useState<StorageReport | null>(null);
  const read = useCallback(() => { api.storageReport().then(setReport).catch(() => {}); }, []);
  // A different model in use changes what is left over: read again at once,
  // so the confirm box never names yesterday's list.
  const modelId = settings.asr.model_id;
  useEffect(() => { read(); }, [read, modelId]);
  useEffect(() => {
    read();
    const timer = poll ? setInterval(read, POLL_MS) : undefined;
    window.addEventListener(STORAGE_CHANGED, read);
    return () => {
      if (timer) clearInterval(timer);
      window.removeEventListener(STORAGE_CHANGED, read);
    };
  }, [read, poll]);
  return { report, refresh: read };
}

/** The one cleaning action every button uses (rail, Settings, "What's new"). */
export async function cleanStorage(keys: string[]): Promise<CleanResult> {
  try {
    return await api.storageClean(keys);
  } finally {
    window.dispatchEvent(new Event(STORAGE_CHANGED));
  }
}

export async function deleteRecordings(): Promise<CleanResult> {
  try {
    return await api.storageDeleteRecordings();
  } finally {
    window.dispatchEvent(new Event(STORAGE_CHANGED));
  }
}
