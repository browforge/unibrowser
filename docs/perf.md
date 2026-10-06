# Performance baseline (local)

Record numbers on a quiet Linux machine with the release desktop binary. Re-run after large viewer or startup changes.

## How to measure

```bash
./scripts/measure-perf-baseline.sh
```

The script prints JSON lines for each scenario. Paste results into the table below with date and git SHA.

| Metric | Method | Result | Notes |
| --- | --- | --- | --- |
| Cold start | Time from process spawn until embedded WebDriver `/status` `ready: true` | _pending_ | Under xvfb; see `scripts/measure-perf-baseline.sh` |
| Idle RAM | RSS from `/proc` after 10s idle | _pending_ | |
| Open 100MB text | `open_path` on generated fixture | _pending_ | |
| Open 4K PNG | `open_path` on `examples/demo-pack` or generated 4K | _pending_ | |

## Flags

Anything above **3s** cold start, **800MB** idle RSS on an empty shell, or **>5s** to first paint on 100MB text deserves a look (not necessarily a blocker for blitz).
