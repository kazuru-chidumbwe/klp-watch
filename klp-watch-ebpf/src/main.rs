#![no_std]
#![no_main]

use aya_ebpf::{
    helpers::{bpf_get_current_comm, bpf_get_current_pid_tgid, bpf_ktime_get_ns},
    macros::{kprobe, map},
    maps::RingBuf,
    programs::ProbeContext,
};
use klp_watch_common::Event;

#[map]
static EVENTS: RingBuf = RingBuf::with_byte_size(64 * 1024, 0);

#[inline(always)]
fn emit(kind: u32) -> u32 {
    if let Some(mut slot) = EVENTS.reserve::<Event>(0) {
        #[allow(unused_unsafe)]
        let ts_ns = unsafe { bpf_ktime_get_ns() };
        slot.write(Event {
            ts_ns,
            kind,
            pid: (bpf_get_current_pid_tgid() >> 32) as u32,
            comm: bpf_get_current_comm().unwrap_or([0u8; 16]),
        });
        slot.submit(0);
    }
    0
}

// Program names match klp-watch-common::PROBES by position.
#[kprobe]
pub fn on_enable_patch(_ctx: ProbeContext) -> u32 {
    emit(0)
}

#[kprobe]
pub fn on_init_transition(_ctx: ProbeContext) -> u32 {
    emit(1)
}

#[kprobe]
pub fn on_start_transition(_ctx: ProbeContext) -> u32 {
    emit(2)
}

#[kprobe]
pub fn on_complete_transition(_ctx: ProbeContext) -> u32 {
    emit(3)
}

#[kprobe]
pub fn on_force_transition(_ctx: ProbeContext) -> u32 {
    emit(4)
}

#[kprobe]
pub fn on_reverse_transition(_ctx: ProbeContext) -> u32 {
    emit(5)
}

#[kprobe]
pub fn on_cancel_transition(_ctx: ProbeContext) -> u32 {
    emit(6)
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[unsafe(link_section = "license")]
#[unsafe(no_mangle)]
static LICENSE: [u8; 13] = *b"Dual MIT/GPL\0";
