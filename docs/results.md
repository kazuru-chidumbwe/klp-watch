# Tested kernels

One row per kernel and test. "Attached" counts the probed functions that attached.

| Date | Kernel | Distribution | Attached | Test | Result |
| --- | --- | --- | --- | --- | --- |
| 2026-11-07 | 6.12.111+deb13-amd64 | Debian 13 | 7 of 7 | `scripts/smoke.sh` with `livepatch-sample` | enable: start to complete 1.06 s; disable: 1.92 s; sysfs agrees |
