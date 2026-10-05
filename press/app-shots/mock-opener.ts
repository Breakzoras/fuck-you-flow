// Stand-in for "@tauri-apps/plugin-opener": links open nothing in the demo.

export async function openUrl(_url: string | URL, _openWith?: string): Promise<void> {}

export async function openPath(_path: string, _openWith?: string): Promise<void> {}

export async function revealItemInDir(_path: string): Promise<void> {}
