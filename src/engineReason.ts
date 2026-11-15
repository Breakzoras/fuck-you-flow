// The words a person sees when the speech engine dies at start.
//
// The engine is a separate program, so on Windows its death shows only as an
// exit code inside the engine message ("whisper-server exited early (exit
// code: 0xc000001d)"). On 14 November 2026 a user on a 2012 laptop saw the app
// stay still and wrote "it does nothing". This turns the code into a sentence
// the screen can show. The same table lives in src-tauri/src/diagnose.rs for
// the problem report; keep the two in step.

const CODES: [hex: string, dec: string, key: string][] = [
  ["0xc000001d", "3221225501", "engine_exit_cpu"],
  ["0xc0000135", "3221225781", "engine_exit_dll"],
  ["0xc000007b", "3221225595", "engine_exit_dll"],
  ["0xc0000142", "3221225794", "engine_exit_runtime"],
  ["0xc0000017", "3221225495", "engine_exit_memory"],
  ["0xc0000005", "3221225477", "engine_exit_crash"],
  ["0xc0000409", "3221226505", "engine_exit_crash"],
  ["0xc00000fd", "3221225725", "engine_exit_crash"],
];

/** The i18n key that explains an engine message, when the message carries a known exit code. */
export function engineExitReason(message: string | null | undefined): string | null {
  if (!message) return null;
  const m = message.toLowerCase();
  for (const [hex, dec, key] of CODES) {
    if (m.includes(hex) || m.includes(dec)) return key;
  }
  return null;
}
