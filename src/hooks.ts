import { createContext, useContext } from "react";
import { Settings } from "./api";
import { t as translate, Strings } from "./i18n";

export interface Ctx {
  settings: Settings;
  setSettings: (s: Settings) => Promise<void>;
  lang: string;
  t: (key: keyof Strings, vars?: Record<string, string | number>) => string;
  toast: (msg: string, kind?: "ok" | "err") => void;
}

export const AppCtx = createContext<Ctx | null>(null);

export function useApp(): Ctx {
  const c = useContext(AppCtx);
  if (!c) throw new Error("no ctx");
  return c;
}

export function makeT(lang: string) {
  return (key: keyof Strings, vars?: Record<string, string | number>) => translate(lang, key, vars);
}
