//! What kind of machine this is: graphics cards, memory, cores. Read once at
//! startup and used to pick the speech engine backend, the model and the
//! thread count, so a fresh install lands on sensible defaults without the
//! user knowing what CUDA or Vulkan are.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct GpuInfo {
    pub name: String,
    /// "nvidia", "amd", "intel" or "other"
    pub vendor: String,
    pub vram_mb: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MachineProfile {
    pub gpus: Vec<GpuInfo>,
    pub logical_cores: usize,
    pub ram_mb: u64,
    /// The Vulkan loader is present (any modern graphics driver installs it).
    pub vulkan_runtime: bool,
    /// The NVIDIA driver is present.
    pub cuda_driver: bool,
}

impl MachineProfile {
    /// The card worth using for speech: the one with the most memory that is a
    /// real adapter (the Windows software renderer is skipped at detection).
    pub fn best_gpu(&self) -> Option<&GpuInfo> {
        self.gpus.iter().max_by_key(|g| g.vram_mb)
    }

    /// Threads for the CPU side of the engine. On the 8-core/16-thread Ryzen
    /// used for development, 12 threads beat both 8 and 16 by about 15%.
    pub fn engine_threads(&self) -> u32 {
        ((self.logical_cores * 3 / 4) as u32).clamp(4, 16)
    }

    pub fn summary(&self) -> String {
        let gpu = match self.best_gpu() {
            Some(g) => format!("{} ({} MB, {})", g.name, g.vram_mb, g.vendor),
            None => "no graphics card".into(),
        };
        format!("{gpu}; {} logical cores; {} MB RAM; vulkan {}; nvidia driver {}", self.logical_cores, self.ram_mb, self.vulkan_runtime, self.cuda_driver)
    }
}

pub fn detect() -> MachineProfile {
    MachineProfile {
        gpus: gpus(),
        logical_cores: std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4),
        ram_mb: ram_mb(),
        vulkan_runtime: system32("vulkan-1.dll"),
        cuda_driver: system32("nvcuda.dll"),
    }
}

fn system32(dll: &str) -> bool {
    let sys = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into());
    std::path::Path::new(&sys).join("System32").join(dll).exists()
}

#[cfg(windows)]
fn ram_mb() -> u64 {
    use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    let mut st = MEMORYSTATUSEX { dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32, ..Default::default() };
    unsafe {
        if GlobalMemoryStatusEx(&mut st).is_ok() {
            return st.ullTotalPhys / (1024 * 1024);
        }
    }
    0
}

#[cfg(not(windows))]
fn ram_mb() -> u64 {
    0
}

/// Every real display adapter, through DXGI (no driver-specific code).
#[cfg(windows)]
fn gpus() -> Vec<GpuInfo> {
    use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE};
    let mut out = Vec::new();
    unsafe {
        let Ok(factory) = CreateDXGIFactory1::<IDXGIFactory1>() else { return out };
        let mut i = 0u32;
        while let Ok(adapter) = factory.EnumAdapters1(i) {
            i += 1;
            let Ok(desc) = adapter.GetDesc1() else { continue };
            if desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32 != 0 {
                continue;
            }
            let name = String::from_utf16_lossy(&desc.Description).trim_end_matches('\0').to_string();
            if name.contains("Microsoft Basic Render") {
                continue;
            }
            let vendor = match desc.VendorId {
                0x10DE => "nvidia",
                0x1002 | 0x1022 => "amd",
                0x8086 => "intel",
                _ => "other",
            };
            out.push(GpuInfo { name, vendor: vendor.into(), vram_mb: desc.DedicatedVideoMemory as u64 / (1024 * 1024) });
        }
    }
    out
}

#[cfg(not(windows))]
fn gpus() -> Vec<GpuInfo> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(gpus: Vec<GpuInfo>, cores: usize) -> MachineProfile {
        MachineProfile { gpus, logical_cores: cores, ram_mb: 16_000, vulkan_runtime: true, cuda_driver: false }
    }

    #[test]
    fn threads_follow_the_measured_sweet_spot() {
        assert_eq!(profile(vec![], 16).engine_threads(), 12);
        assert_eq!(profile(vec![], 8).engine_threads(), 6);
        assert_eq!(profile(vec![], 4).engine_threads(), 4);
        assert_eq!(profile(vec![], 32).engine_threads(), 16);
    }

    #[test]
    fn best_gpu_is_the_one_with_most_memory() {
        let p = profile(
            vec![
                GpuInfo { name: "iGPU".into(), vendor: "intel".into(), vram_mb: 128 },
                GpuInfo { name: "RX".into(), vendor: "amd".into(), vram_mb: 8192 },
            ],
            16,
        );
        assert_eq!(p.best_gpu().unwrap().name, "RX");
        assert!(profile(vec![], 8).best_gpu().is_none());
    }
}
