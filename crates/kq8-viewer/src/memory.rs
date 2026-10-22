//! How much memory the program uses, for the logs (M6 6e-2: a world
//! change must not leave the old world behind). Windows only; elsewhere
//! the line says it is not known.

/// "memory: working set N MB, private N MB" (or "not known").
pub fn line() -> String {
    match usage() {
        Some((working, private)) => format!("memory: working set {} MB, private {} MB", working / (1 << 20), private / (1 << 20)),
        None => "memory: not known".to_string(),
    }
}

/// (working set, private bytes) of this process.
#[cfg(windows)]
fn usage() -> Option<(usize, usize)> {
    // `PROCESS_MEMORY_COUNTERS` (psapi.h), filled by Windows.
    #[repr(C)]
    #[derive(Default)]
    struct Counters {
        cb: u32,
        page_fault_count: u32,
        peak_working_set_size: usize,
        working_set_size: usize,
        quota_peak_paged_pool_usage: usize,
        quota_paged_pool_usage: usize,
        quota_peak_non_paged_pool_usage: usize,
        quota_non_paged_pool_usage: usize,
        pagefile_usage: usize,
        peak_pagefile_usage: usize,
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn GetCurrentProcess() -> isize;
        fn K32GetProcessMemoryInfo(process: isize, counters: *mut Counters, cb: u32) -> i32;
    }
    let mut c = Counters { cb: std::mem::size_of::<Counters>() as u32, ..Default::default() };
    // SAFETY: `c` is a correctly sized, writable `PROCESS_MEMORY_COUNTERS`
    // and `cb` says its size; the pseudo-handle of the current process
    // needs no closing.
    let ok = unsafe { K32GetProcessMemoryInfo(GetCurrentProcess(), &mut c, c.cb) };
    (ok != 0).then_some((c.working_set_size, c.pagefile_usage))
}

#[cfg(not(windows))]
fn usage() -> Option<(usize, usize)> {
    None
}
