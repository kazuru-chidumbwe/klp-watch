//! klp-watch: print Linux livepatch transition events as JSON lines.

use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use aya::{maps::RingBuf, programs::KProbe};
use klp_watch_common::{Event, PROBES};
use log::{debug, warn};
use tokio::{io::unix::AsyncFd, signal};

const PROGRAMS: [&str; 7] = [
    "on_enable_patch",
    "on_init_transition",
    "on_start_transition",
    "on_complete_transition",
    "on_force_transition",
    "on_reverse_transition",
    "on_cancel_transition",
];

fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn wall_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

fn read_trim(p: &Path) -> String {
    fs::read_to_string(p)
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

/// One line per patch currently under /sys/kernel/livepatch, as a cross-check.
fn sysfs_snapshot() {
    let Ok(dir) = fs::read_dir("/sys/kernel/livepatch") else {
        println!("{{\"event\":\"sysfs\",\"error\":\"no /sys/kernel/livepatch\"}}");
        return;
    };
    for entry in dir.flatten() {
        let p = entry.path();
        println!(
            "{{\"event\":\"sysfs\",\"wall_ms\":{},\"patch\":{},\"enabled\":{},\"transition\":{}}}",
            wall_ms(),
            json_str(&entry.file_name().to_string_lossy()),
            json_str(&read_trim(&p.join("enabled"))),
            json_str(&read_trim(&p.join("transition"))),
        );
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    // Needed on kernels without memcg-based BPF memory accounting.
    let rlim = libc::rlimit {
        rlim_cur: libc::RLIM_INFINITY,
        rlim_max: libc::RLIM_INFINITY,
    };
    let ret = unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlim) };
    if ret != 0 {
        debug!("remove limit on locked memory failed, ret is: {ret}");
    }

    let mut ebpf = aya::Ebpf::load(aya::include_bytes_aligned!(concat!(
        env!("OUT_DIR"),
        "/klp-watch"
    )))?;

    println!(
        "{{\"event\":\"start\",\"wall_ms\":{},\"kernel\":{}}}",
        wall_ms(),
        json_str(&read_trim(Path::new("/proc/sys/kernel/osrelease")))
    );

    for (prog, func) in PROGRAMS.iter().zip(PROBES.iter()) {
        let program: &mut KProbe = ebpf.program_mut(prog).unwrap().try_into()?;
        program.load()?;
        let attached = match program.attach(*func, 0) {
            Ok(_) => true,
            Err(e) => {
                warn!("attach {func} failed: {e}");
                false
            }
        };
        println!(
            "{{\"event\":\"probe\",\"function\":{},\"attached\":{attached}}}",
            json_str(func)
        );
    }
    sysfs_snapshot();

    let ring = RingBuf::try_from(ebpf.take_map("EVENTS").unwrap())?;
    // SAFETY: the ring buffer owns its file descriptor, which stays open and unchanged
    // while the AsyncFd owns it.
    let mut fd = unsafe { AsyncFd::register_with_interest(ring, tokio::io::Interest::READABLE) }?;
    let ctrl_c = signal::ctrl_c();
    tokio::pin!(ctrl_c);

    loop {
        tokio::select! {
            _ = &mut ctrl_c => break,
            guard = fd.readable_mut() => {
                let mut guard = guard?;
                let ring = guard.get_inner_mut();
                while let Some(item) = ring.next() {
                    if item.len() < core::mem::size_of::<Event>() {
                        continue;
                    }
                    let ev: Event = unsafe { core::ptr::read_unaligned(item.as_ptr() as *const Event) };
                    let comm_len = ev.comm.iter().position(|&b| b == 0).unwrap_or(ev.comm.len());
                    let comm = String::from_utf8_lossy(&ev.comm[..comm_len]);
                    let name = PROBES.get(ev.kind as usize).copied().unwrap_or("unknown");
                    println!(
                        "{{\"event\":\"klp\",\"function\":{},\"ts_ns\":{},\"wall_ms\":{},\"pid\":{},\"comm\":{}}}",
                        json_str(name), ev.ts_ns, wall_ms(), ev.pid, json_str(&comm)
                    );
                }
                guard.clear_ready();
                // A completed transition changes sysfs; record it next to the event.
                sysfs_snapshot();
            }
        }
    }
    println!("{{\"event\":\"stop\",\"wall_ms\":{}}}", wall_ms());
    Ok(())
}
