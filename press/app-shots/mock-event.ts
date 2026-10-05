// Stand-in for "@tauri-apps/api/event": nothing is ever emitted in the demo.

export type UnlistenFn = () => void;
export interface Event<T> { event: string; id: number; payload: T }
export type EventCallback<T> = (event: Event<T>) => void;

const noop: UnlistenFn = () => {};

export async function listen<T>(_event: string, _handler: EventCallback<T>): Promise<UnlistenFn> {
  return noop;
}

export async function once<T>(_event: string, _handler: EventCallback<T>): Promise<UnlistenFn> {
  return noop;
}

export async function emit(_event: string, _payload?: unknown): Promise<void> {}

export async function emitTo(_target: unknown, _event: string, _payload?: unknown): Promise<void> {}
