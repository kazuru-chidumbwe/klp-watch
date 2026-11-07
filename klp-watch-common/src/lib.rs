#![no_std]

/// Kernel functions klp-watch attaches to, in event-kind order.
pub const PROBES: [&str; 7] = [
    "klp_enable_patch",
    "klp_init_transition",
    "klp_start_transition",
    "klp_complete_transition",
    "klp_force_transition",
    "klp_reverse_transition",
    "klp_cancel_transition",
];

/// One event per probe hit, sent through the ring buffer.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Event {
    /// CLOCK_MONOTONIC nanoseconds (bpf_ktime_get_ns).
    pub ts_ns: u64,
    /// Index into PROBES.
    pub kind: u32,
    /// Thread group id of the task that hit the probe.
    pub pid: u32,
    /// Task command name.
    pub comm: [u8; 16],
}

#[cfg(feature = "user")]
unsafe impl aya::Pod for Event {}
