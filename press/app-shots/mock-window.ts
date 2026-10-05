// Stand-in for "@tauri-apps/api/window". Every method is a quiet no-op.

export enum UserAttentionType {
  Critical = 1,
  Informational = 2,
}

export class LogicalSize { constructor(public width: number, public height: number) {} }
export class PhysicalSize { constructor(public width: number, public height: number) {} }
export class LogicalPosition { constructor(public x: number, public y: number) {} }
export class PhysicalPosition { constructor(public x: number, public y: number) {} }

const W = 1280;
const H = 820;

const win = {
  label: "main",
  show: async () => {},
  hide: async () => {},
  close: async () => {},
  minimize: async () => {},
  unminimize: async () => {},
  maximize: async () => {},
  unmaximize: async () => {},
  setFocus: async () => {},
  requestUserAttention: async (_t?: UserAttentionType | null) => {},
  isVisible: async () => true,
  isFocused: async () => true,
  isMinimized: async () => false,
  isMaximized: async () => false,
  innerSize: async () => new PhysicalSize(W, H),
  outerSize: async () => new PhysicalSize(W, H),
  innerPosition: async () => new PhysicalPosition(0, 0),
  outerPosition: async () => new PhysicalPosition(0, 0),
  scaleFactor: async () => 1,
  setPosition: async (_p: unknown) => {},
  setSize: async (_s: unknown) => {},
  setAlwaysOnTop: async (_on: boolean) => {},
  setIgnoreCursorEvents: async (_on: boolean) => {},
  setTitle: async (_t: string) => {},
  startDragging: async () => {},
  listen: async () => () => {},
  once: async () => () => {},
  onMoved: async () => () => {},
  onResized: async () => () => {},
  onFocusChanged: async () => () => {},
  onCloseRequested: async () => () => {},
};

export function getCurrentWindow() {
  return win;
}

export async function currentMonitor() {
  return { name: "Display 1", size: new PhysicalSize(1920, 1080), position: new PhysicalPosition(0, 0), scaleFactor: 1 };
}

export async function availableMonitors() {
  return [await currentMonitor()];
}
