// The notepad window. It opens by itself the moment a dictation cannot reach
// the window the user was in, and after a sound file has been turned into
// text. Its whole job is that the words are visible and easy to take: shown,
// selected, one button to copy, one to close.

import React, { useEffect, useRef, useState } from "react";
import ReactDOM from "react-dom/client";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./scratch.css";

interface Payload {
  text: string;
  reason: string;
  detail?: string | null;
  source: string;
}

type Dict = Record<string, string>;

const REASONS: Record<string, Dict> = {
  en: {
    not_taken: "That window did not take the words. Click inside a text box and next time they go straight in.",
    window_closed: "The window you were dictating into closed before the words were ready.",
    elevated: "That window runs as administrator, so nothing can be written into it from here.",
    focus_lost: "You moved to another window while the words were being written.",
    insert_failed: "The words could not be written into that window.",
    file: "Here is the text of your recording.",
  },
  el: {
    not_taken: "Εκείνο το παράθυρο δεν πήρε τα λόγια. Κάνε κλικ μέσα σε ένα πλαίσιο κειμένου και την επόμενη φορά μπαίνουν κατευθείαν.",
    window_closed: "Το παράθυρο στο οποίο μιλούσες έκλεισε πριν ετοιμαστούν τα λόγια.",
    elevated: "Εκείνο το παράθυρο τρέχει ως διαχειριστής, οπότε από εδώ δεν γράφεται τίποτα μέσα του.",
    focus_lost: "Πήγες σε άλλο παράθυρο όσο γράφονταν τα λόγια.",
    insert_failed: "Τα λόγια δεν μπόρεσαν να γραφτούν σε εκείνο το παράθυρο.",
    file: "Ορίστε το κείμενο της ηχογράφησής σου.",
  },
};

const UI: Record<string, Dict> = {
  en: {
    title: "Your words",
    safe: "They are on the clipboard as well, so Ctrl+V works anywhere.",
    copy: "Copy",
    copied: "Copied",
    close: "Close",
    from: "From",
    dont_show: "Do not show this window again",
    dont_show_hint: "The words still go to the clipboard, so Ctrl+V keeps working. You can switch it back on under Settings, Insertion.",
  },
  el: {
    title: "Τα λόγια σου",
    safe: "Βρίσκονται και στο πρόχειρο, οπότε το Ctrl+V δουλεύει παντού.",
    copy: "Αντιγραφή",
    copied: "Αντιγράφηκε",
    close: "Κλείσιμο",
    from: "Από",
    dont_show: "Να μην ξαναεμφανιστεί αυτό το παράθυρο",
    dont_show_hint: "Τα λόγια πάνε ούτως ή άλλως στο πρόχειρο, οπότε το Ctrl+V δουλεύει κανονικά. Το ξανανοίγεις από τις Ρυθμίσεις, Εισαγωγή κειμένου.",
  },
};

async function fetchLook(): Promise<{ lang: string; theme: string }> {
  try {
    const s = await invoke<{ general: { ui_language: string; theme: string } }>("get_settings");
    return {
      lang: s.general.ui_language in UI ? s.general.ui_language : "en",
      theme: s.general.theme ?? "system",
    };
  } catch {
    return { lang: "en", theme: "system" };
  }
}

/// The window has to follow the same theme as the dashboard. It used to be
/// painted dark whatever the user had chosen, so on a light desktop a black
/// box jumped out in the middle of the screen.
function paintTheme(theme: string) {
  const dark = theme === "dark" || (theme === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches);
  document.documentElement.setAttribute("data-theme", dark ? "dark" : "light");
}

function Scratch() {
  const [p, setP] = useState<Payload | null>(null);
  const [lang, setLang] = useState("en");
  const [copied, setCopied] = useState(false);
  // The box is unticked again for every new text. The window is only hidden,
  // never destroyed, so without this it would come back already ticked and a
  // click to untick it would do nothing at all.
  const [never, setNever] = useState(false);
  const box = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    fetchLook().then(({ lang: l, theme }) => { setLang(l); paintTheme(theme); });
    // A webview that reloads misses the event that opened it, so ask for the
    // last thing the app wanted to show.
    invoke<Payload | null>("scratch_last").then((last) => last && setP(last)).catch(() => {});
    const un = listen<Payload>("lalia://scratch", (ev) => {
      setP(ev.payload);
      setCopied(false);
      setNever(false);
      fetchLook().then(({ lang: l, theme }) => { setLang(l); paintTheme(theme); });
    });
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const onScheme = () => { fetchLook().then(({ theme }) => paintTheme(theme)); };
    mq.addEventListener("change", onScheme);
    return () => {
      un.then((f) => f());
      mq.removeEventListener("change", onScheme);
    };
  }, []);

  // Selected on arrival: one Ctrl+C, or one drag, and the words are yours.
  useEffect(() => {
    if (p && box.current) {
      box.current.focus();
      box.current.select();
    }
  }, [p]);

  const t = UI[lang] ?? UI.en;
  const reasons = REASONS[lang] ?? REASONS.en;

  const copy = () => {
    const el = box.current;
    if (!el) return;
    el.focus();
    el.select();
    try {
      document.execCommand("copy");
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1600);
    } catch {
      // The text is already on the clipboard from the app side; nothing to do.
    }
  };

  const close = () => {
    getCurrentWindow().hide();
  };

  // Switching it off at the moment it is in the way, which is the only moment
  // anyone wants to. The same switch lives in Settings, Insertion.
  const neverAgain = (on: boolean) => {
    setNever(on);
    if (!on) return;
    invoke("set_notepad_when_lost", { enabled: false }).catch(() => {});
    getCurrentWindow().hide();
  };

  useEffect(() => {
    const onKey = (ev: KeyboardEvent) => {
      if (ev.key === "Escape") close();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  return (
    <div className="wrap">
      <div className="reason">
        {reasons[p?.reason ?? "file"] ?? reasons.file}
        {p?.source === "file" && p?.detail ? ` ${t.from}: ${p.detail}` : ""}
        <div className="said">{t.safe}</div>
      </div>
      <textarea ref={box} value={p?.text ?? ""} onChange={(ev) => setP({ ...(p as Payload), text: ev.target.value })} spellCheck={false} aria-label={t.title} />
      <div className="row">
        <button className="primary" onClick={copy}>
          {copied ? t.copied : t.copy}
        </button>
        <div className="grow" />
        <button onClick={close}>{t.close}</button>
      </div>
      {p?.source !== "file" && (
        <label className="never">
          <input type="checkbox" checked={never} onChange={(ev) => neverAgain(ev.target.checked)} />
          <span>
            {t.dont_show}
            <em>{t.dont_show_hint}</em>
          </span>
        </label>
      )}
    </div>
  );
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <Scratch />
  </React.StrictMode>
);
