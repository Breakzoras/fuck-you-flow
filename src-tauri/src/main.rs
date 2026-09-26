// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Linux: run as an X11 client, under XWayland on a Wayland desktop, so the
    // clipboard, the focused window and the overlay position all work the same
    // way everywhere. The DMA-BUF switch avoids the blank white window WebKitGTK
    // shows on some NVIDIA drivers. A value the user set wins.
    #[cfg(target_os = "linux")]
    {
        if std::env::var_os("GDK_BACKEND").is_none() {
            std::env::set_var("GDK_BACKEND", "x11");
        }
        if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
    }
    fuckyouflow_lib::run()
}
