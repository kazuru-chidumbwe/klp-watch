# klp-watch

klp-watch traces Linux kernel livepatch transitions with eBPF and prints one JSON line per event: when a patch is enabled, when its transition starts, and when it completes, is reversed, cancelled or forced. Each event carries a kernel timestamp and the task that triggered it, and a snapshot of `/sys/kernel/livepatch` is printed next to it as a cross-check. It only observes; it never changes a patch.

It is written in Rust with [Aya](https://aya-rs.dev).

License: MIT or Apache-2.0 for user space; the eBPF program is dual MIT/GPL. See [`LICENSE-MIT`](LICENSE-MIT), [`LICENSE-APACHE`](LICENSE-APACHE) and [`LICENSE-GPL2`](LICENSE-GPL2).

## Try it

Needs Linux with `CONFIG_LIVEPATCH`, root, Rust stable and nightly (with `rust-src`), and [bpf-linker](https://github.com/aya-rs/bpf-linker).

```bash
git clone https://github.com/kazuru-chidumbwe/klp-watch.git
cd klp-watch
cargo build --release
sudo ./target/release/klp-watch
```

Leave it running and load or disable a livepatch in another terminal. Stop it with Ctrl-C.

### Smoke test

`scripts/smoke.sh` loads the kernel's `livepatch-sample` module, disables it and unloads it while klp-watch records. Build the sample from `samples/livepatch/livepatch-sample.c` against your kernel's headers first, and point `MOD` at it. Run it in a test VM or lab machine, not on a production host.

```bash
MOD=/path/to/livepatch-sample.ko scripts/smoke.sh /tmp/klp-watch.jsonl
```

## Output

```json
{"event":"klp","function":"klp_enable_patch","ts_ns":301212889604979,"wall_ms":1794040021118,"pid":20473,"comm":"insmod"}
{"event":"klp","function":"klp_start_transition","ts_ns":301212891655311,"wall_ms":1794040021121,"pid":20473,"comm":"insmod"}
{"event":"klp","function":"klp_complete_transition","ts_ns":301213954999633,"wall_ms":1794040022184,"pid":13753,"comm":"kworker/4:0"}
{"event":"sysfs","wall_ms":1794040022184,"patch":"livepatch_sample","enabled":"1","transition":"0"}
```

`ts_ns` is `CLOCK_MONOTONIC`, so the difference between `klp_start_transition` and `klp_complete_transition` is the transition time. Probes fire on function entry, so a sysfs line printed with a `klp_complete_transition` event can still show the old state.

Probed functions: `klp_enable_patch`, `klp_init_transition`, `klp_start_transition`, `klp_complete_transition`, `klp_force_transition`, `klp_reverse_transition`, `klp_cancel_transition`. Kernels where a function is missing or inlined print `"attached":false` for it and keep going.

Tested kernels: [`docs/results.md`](docs/results.md).

## Not yet

- The patch name inside each event (today it comes from the sysfs line).
- Which tasks hold up a transition.
- Whether the patched version of a function is the one that runs.
