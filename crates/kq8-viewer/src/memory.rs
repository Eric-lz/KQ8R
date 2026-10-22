//! How much memory the program uses, for the logs (M6 6e-2: a world
//! change must not leave the old world behind). Windows only; elsewhere
//! the line says it is not known.
//!
//! The allocator is `mimalloc` (chosen by the user 2026-10-22: with the
//! Windows one, reading a world slowed with every world change, Daventry
//! 3.5 s fresh, 14.6 s after a tour of all worlds; with `mimalloc`
//! 3.1-3.7 s throughout). Its purge delay is 0: memory it no longer uses
//! goes back to the system at once instead of after 1 s (the tour's end
//! in Daventry: 3.0 GB working set, 5.1 GB private, against 3.5 and 5.4).

/// `mi_option_purge_delay` in `mimalloc.h` (v3; the Rust bindings don't
/// name it).
const PURGE_DELAY: libmimalloc_sys::mi_option_t = 15;
/// `mi_option_purge_decommits` (1, its default: give the pages back, not
/// only mark them unused).
const PURGE_DECOMMITS: libmimalloc_sys::mi_option_t = 5;

/// Sets the allocator's options; first thing in `main`.
pub fn set_allocator() {
    // SAFETY: plain setters of mimalloc's global options.
    unsafe { libmimalloc_sys::mi_option_set(PURGE_DELAY, 0) };
}

/// "allocator: mimalloc 3.3.2, purge delay 0 ms, purge decommits 1".
pub fn allocator_line() -> String {
    // SAFETY: plain getters of mimalloc's version and global options.
    let (v, delay, decommits) = unsafe {
        (libmimalloc_sys::mi_version(), libmimalloc_sys::mi_option_get(PURGE_DELAY), libmimalloc_sys::mi_option_get(PURGE_DECOMMITS))
    };
    format!("allocator: mimalloc {}.{}.{}, purge delay {delay} ms, purge decommits {decommits}", v / 10000, v / 100 % 100, v % 100)
}

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
