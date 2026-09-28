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

/// Whether startup should put the engine back on the graphics card.
///
/// Kept as its own function because the answer decides how fast the app is on
/// a stranger's machine and it has to be testable without a graphics card in
/// the room. The first AMD tester, on 7 September 2026, ran on the processor
/// with the engine reported as missing until he found the two switches
/// himself; every condition below is one half of why that was allowed.
pub fn should_switch_to_gpu(use_gpu: bool, backend: &str, chosen_by_user: bool, has_card: bool, vulkan_runtime: bool, vulkan_build: bool) -> bool {
    if chosen_by_user {
        return false;
    }
    let on_the_processor = !use_gpu || backend == "cpu";
    on_the_processor && has_card && vulkan_runtime && vulkan_build
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

/// How full the card is at this moment, as opposed to how big it is. A card
/// that is full does not refuse the speech model: Windows accepts it and pages
/// most of it back out to system RAM, where the same work runs roughly thirty
/// times slower. That is what turned a one second dictation into a thirty
/// second one on 7 September 2026, with the model asking for the same 1960 MB
/// it had asked for all morning and only 90 MB of it staying on the card.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct GpuMemory {
    /// The card's own memory.
    pub total_mb: u64,
    /// Held on the card right now, by every program together.
    pub used_mb: u64,
    /// What is left. What a speech model has to fit inside.
    pub free_mb: u64,
}

/// Read live, on every poll. Unlike [`detect`], this changes minute to minute.
///
/// The obvious source, DXGI's `QueryVideoMemoryInfo`, is the wrong one: it
/// reports the budget Windows is willing to promise a process, which on
/// 7 September 2026 read 7249 MB on a card `nvidia-smi` showed with 1897 MB
/// free. A promise is not a measurement. The performance counter below agreed
/// with `nvidia-smi` to within 20 MB in the same second, and unlike
/// `nvidia-smi` it is there on AMD and Intel too.
#[cfg(windows)]
pub fn gpu_memory() -> Option<GpuMemory> {
    let (total_mb, luid) = card()?;
    let used_mb = adapter_dedicated_usage_mb(&luid)?;
    Some(GpuMemory { total_mb, used_mb, free_mb: total_mb.saturating_sub(used_mb) })
}

/// The card [`MachineProfile::best_gpu`] would pick, with the identifier the
/// performance counters name it by: `luid_0xHHHHHHHH_0xLLLLLLLL`.
#[cfg(windows)]
fn card() -> Option<(u64, String)> {
    use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE};
    unsafe {
        let factory = CreateDXGIFactory1::<IDXGIFactory1>().ok()?;
        let mut best: Option<(u64, String)> = None;
        let mut i = 0u32;
        while let Ok(adapter) = factory.EnumAdapters1(i) {
            i += 1;
            let Ok(desc) = adapter.GetDesc1() else { continue };
            if desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32 != 0 {
                continue;
            }
            if String::from_utf16_lossy(&desc.Description).contains("Microsoft Basic Render") {
                continue;
            }
            let mb = desc.DedicatedVideoMemory as u64 / (1024 * 1024);
            let luid = format!("luid_0x{:08x}_0x{:08x}", desc.AdapterLuid.HighPart, desc.AdapterLuid.LowPart);
            if best.as_ref().is_none_or(|(b, _)| mb > *b) {
                best = Some((mb, luid));
            }
        }
        best
    }
}

/// How much of one program's graphics memory is still on the card, and how much
/// Windows has pushed out to system RAM. This is the honest health signal for
/// speech: the model is not smaller when the card fills up, it is *elsewhere*,
/// and the work then crawls across the bus instead of running on the card.
///
/// Returns `(on_card_mb, pushed_out_mb)` for the process, or `None` when the
/// process holds no graphics memory at all.
#[cfg(windows)]
pub fn process_gpu_memory(pid: u32) -> Option<(u64, u64)> {
    let prefix = format!("pid_{pid}_");
    let on_card = pdh_sum_mb("\\GPU Process Memory(*)\\Local Usage", &prefix)?;
    let pushed_out = pdh_sum_mb("\\GPU Process Memory(*)\\Non Local Usage", &prefix).unwrap_or(0);
    Some((on_card, pushed_out))
}

#[cfg(not(windows))]
pub fn process_gpu_memory(_pid: u32) -> Option<(u64, u64)> {
    None
}

/// `\GPU Adapter Memory(<luid>_phys_N)\Dedicated Usage`, summed over the
/// segments of one card.
#[cfg(windows)]
fn adapter_dedicated_usage_mb(luid: &str) -> Option<u64> {
    pdh_sum_mb("\\GPU Adapter Memory(*)\\Dedicated Usage", luid)
}

/// Sums one wildcard performance counter over the instances whose name starts
/// with `instance_prefix`, in MB. Counters are added by their English names so
/// the paths still resolve on a Windows installed in another language.
#[cfg(windows)]
fn pdh_sum_mb(counter_path: &str, instance_prefix: &str) -> Option<u64> {
    use windows::core::PCWSTR;
    use windows::Win32::System::Performance::{
        PdhAddEnglishCounterW, PdhCloseQuery, PdhCollectQueryData, PdhGetFormattedCounterArrayW, PdhOpenQueryW,
        PDH_FMT_COUNTERVALUE_ITEM_W, PDH_FMT_LARGE,
    };
    const OK: u32 = 0;
    const MORE_DATA: u32 = 0x800007D2;

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    unsafe {
        let mut query = Default::default();
        if PdhOpenQueryW(PCWSTR::null(), 0, &mut query) != OK {
            return None;
        }
        // Every exit from here on has to close the query, so the body runs once
        // inside a closure and the handle is released after it either way.
        let read = || -> Option<u64> {
            let path = wide(counter_path);
            let mut counter = Default::default();
            let st = PdhAddEnglishCounterW(query, PCWSTR(path.as_ptr()), 0, &mut counter);
            if st != OK {
                tracing::debug!("gpu counter: add failed 0x{st:08x}");
                return None;
            }
            let st = PdhCollectQueryData(query);
            if st != OK {
                tracing::debug!("gpu counter: collect failed 0x{st:08x}");
                return None;
            }
            let mut bytes = 0u32;
            let mut items = 0u32;
            let st = PdhGetFormattedCounterArrayW(counter, PDH_FMT_LARGE, &mut bytes, &mut items, None);
            if st != MORE_DATA {
                tracing::debug!("gpu counter: sizing said 0x{st:08x}, wanted 0x{MORE_DATA:08x}");
                return None;
            }
            let mut buf = vec![0u8; bytes as usize];
            let head = buf.as_mut_ptr() as *mut PDH_FMT_COUNTERVALUE_ITEM_W;
            let st = PdhGetFormattedCounterArrayW(counter, PDH_FMT_LARGE, &mut bytes, &mut items, Some(head));
            if st != OK {
                tracing::debug!("gpu counter: read failed 0x{st:08x}");
                return None;
            }
            // Windows spells the adapter identifier with uppercase hex here and
            // DXGI hands it over as a number, so the two are compared folded.
            let want = instance_prefix.to_ascii_lowercase();
            let mut total = 0i64;
            let mut matched = false;
            for k in 0..items as usize {
                let item = &*head.add(k);
                let Ok(name) = item.szName.to_string() else { continue };
                if !name.to_ascii_lowercase().starts_with(&want) {
                    continue;
                }
                matched = true;
                total += item.FmtValue.Anonymous.largeValue;
            }
            matched.then(|| (total.max(0) as u64) / (1024 * 1024))
        };
        let out = read();
        let _ = PdhCloseQuery(query);
        out
    }
}

#[cfg(not(windows))]
pub fn gpu_memory() -> Option<GpuMemory> {
    None
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

    /// Prints the live reading so it can be held against `nvidia-smi` in the
    /// same second. Asserts only what must hold on any machine with a card.
    #[test]
    #[cfg(windows)]
    fn gpu_memory_reads_a_live_usage_that_adds_up() {
        let _ = tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG).with_test_writer().try_init();
        let Some(m) = gpu_memory() else {
            eprintln!("no card with a memory counter on this machine, nothing to check");
            return;
        };
        eprintln!("card {} MB | used now {} MB | free {} MB", m.total_mb, m.used_mb, m.free_mb);
        assert!(m.total_mb > 0, "a real card reports its size");
        assert!(m.used_mb <= m.total_mb, "no card holds more than it has");
        assert_eq!(m.free_mb, m.total_mb - m.used_mb, "the three numbers agree");

        // Point this at a live whisper-server to see how much of the speech
        // model is still on the card:  LALIA_PROBE_PID=1234 cargo test hw::
        if let Some(pid) = std::env::var("LALIA_PROBE_PID").ok().and_then(|v| v.parse::<u32>().ok()) {
            match process_gpu_memory(pid) {
                Some((on_card, pushed_out)) => {
                    eprintln!("pid {pid}: {on_card} MB on the card, {pushed_out} MB pushed out to RAM")
                }
                None => eprintln!("pid {pid} holds no graphics memory"),
            }
        }
    }
}

/// The language Windows itself is set to, as a BCP-47 tag like "el-GR" or
/// "en-GB", lowercased. Used once, on a fresh install, to choose the interface
/// language and the dictation language. Falls back to "en" when Windows will
/// not say.
pub fn user_locale() -> String {
    use windows::Win32::Globalization::GetUserDefaultLocaleName;
    let mut buf = [0u16; 85];
    let n = unsafe { GetUserDefaultLocaleName(&mut buf) };
    if n <= 1 {
        return "en".into();
    }
    String::from_utf16_lossy(&buf[..(n as usize - 1)]).to_lowercase()
}

#[cfg(test)]
mod gpu_choice_tests {
    use super::should_switch_to_gpu;

    /// The case that started this: an AMD machine with the card switched off.
    #[test]
    fn amd_with_the_card_switched_off_is_corrected() {
        assert!(should_switch_to_gpu(false, "auto", false, true, true, true));
    }

    /// Acceleration pinned to the processor is the same failure by another name.
    #[test]
    fn backend_pinned_to_cpu_is_corrected() {
        assert!(should_switch_to_gpu(true, "cpu", false, true, true, true));
    }

    /// A user who turned the card off meant it. We never argue.
    #[test]
    fn a_deliberate_choice_is_never_overruled() {
        assert!(!should_switch_to_gpu(false, "cpu", true, true, true, true));
    }

    /// Nothing to switch to: no card, no Vulkan on the machine, or no Vulkan
    /// build shipped with this installation.
    #[test]
    fn nothing_to_switch_to_stays_as_it_is() {
        assert!(!should_switch_to_gpu(false, "auto", false, false, true, true), "no card");
        assert!(!should_switch_to_gpu(false, "auto", false, true, false, true), "no Vulkan runtime");
        assert!(!should_switch_to_gpu(false, "auto", false, true, true, false), "no Vulkan build");
    }

    /// A machine already on the card is left alone.
    #[test]
    fn a_working_setup_is_left_alone() {
        assert!(!should_switch_to_gpu(true, "auto", false, true, true, true));
        assert!(!should_switch_to_gpu(true, "vulkan", false, true, true, true));
    }
}
