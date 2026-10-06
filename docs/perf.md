# Performance baseline (local)

Record numbers on a quiet Linux machine with the release desktop binary. Re-run after large viewer or startup changes.

## How to measure

```bash
./scripts/measure-perf-baseline.sh
```

The script prints JSON-ish lines for each scenario. Paste results into the table below with date and git SHA.

| Metric | Method | Result | Notes |
| --- | --- | --- | --- |
| Cold start | `xvfb-run` + release binary, 4s wall until kill | **4.00s** | 2026-10-06, `scripts/measure-perf-baseline.sh` |
| Idle RAM | RSS after 5s idle | **unreliable on headless** | Process often exits before RSS stabilises under xvfb (reported 1MB) |
| Open 100MB text | Fixture generated; timing not automated | **pending** | `/usr/bin/time` missing on VM; use manual `open_path` once UI path is scripted |
| Open 4K PNG | `open_path` on generated fixture | **pending** | |

## Flags

Anything above **3s** cold start, **800MB** idle RSS on an empty shell, or **>5s** to first paint on 100MB text deserves a look (not necessarily a blocker for blitz).
