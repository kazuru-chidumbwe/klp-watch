#!/usr/bin/env bash
# Smoke test: load the kernel's livepatch sample, disable it, unload it; klp-watch records events.
set -u
OUT=${1:-/tmp/klp-watch-smoke.jsonl}
BIN=$(dirname "$0")/../target/release/klp-watch
MOD=${MOD:-$HOME/src/klp-sample/livepatch-sample.ko}
P=/sys/kernel/livepatch/livepatch_sample
sudo "$BIN" > "$OUT" 2>"$OUT.err" & W=$!
sleep 2
sudo /sbin/insmod "$MOD" || { echo "insmod failed"; sudo kill -INT $W; exit 1; }
for i in $(seq 1 30); do [ "$(cat $P/transition 2>/dev/null)" = 0 ] && break; sleep 1; done
echo "cmdline while patched: $(cat /proc/cmdline)" >&2
echo 0 | sudo tee $P/enabled >/dev/null
for i in $(seq 1 30); do [ -d $P ] || break; sleep 1; done
sudo /sbin/rmmod livepatch_sample
sleep 1
sudo pkill -INT -x klp-watch; wait $W 2>/dev/null
cat "$OUT"
