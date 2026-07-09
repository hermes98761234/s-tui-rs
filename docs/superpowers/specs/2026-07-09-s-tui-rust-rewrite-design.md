# s-tui-rs — Rust rewrite of s-tui

Design spec, 2026-07-09. Source project: https://github.com/amanusk/s-tui (Python, urwid + psutil).
Decisions below are the recommended options, chosen without user review per explicit instruction.

## Goal

A terminal UI that monitors CPU temperature, frequency, utilization, power, and fan speed as
scrolling graphs plus text summaries, with built-in CPU stress testing — a functional port of
s-tui to Rust, shipped as a single static-ish binary for Linux, macOS, and Windows.

## Stack

| Concern | Choice | Why |
|---|---|---|
| TUI | `ratatui` + `crossterm` | De-facto standard, cross-platform, has Sparkline/Chart/BarChart widgets |
| Sensors | `sysinfo` crate + direct sysfs reads on Linux | sysinfo covers util/freq/temps cross-platform; sysfs for RAPL power, throttle counts, fans |
| CLI | `clap` (derive) | Standard |
| Config | `serde` + `toml`, stored at `~/.config/s-tui-rs/config.toml` (`directories` crate for paths) | Rewrite may break INI compat; TOML is the Rust norm |
| Errors | `anyhow` (bin-level), per-source soft-failure flags | UI must never crash on a sensor read error |
| Export | `serde_json` for JSON; hand-rolled CSV writer (one format, no `csv` crate needed) | Minimal deps |

## Architecture

```
src/
├── main.rs        # clap parsing; dispatch: TUI mode vs one-shot (--json/--terminal)
├── app.rs         # App state: sources, per-sensor history (VecDeque, 1000 samples), tick loop
├── ui.rs          # ratatui rendering: graph panes, summary sidebar, help/sensors/stress menus
├── sources/
│   ├── mod.rs     # trait Source { name, unit, read() -> Vec<Reading>, available() }
│   ├── temp.rs    # sysinfo Components; alert threshold (default 80°C, --t-thresh)
│   ├── freq.rs    # sysinfo per-core MHz; Linux: throttle counts from sysfs thermal_throttle
│   ├── util.rs    # sysinfo per-core % + average
│   ├── power.rs   # Linux only: Intel RAPL energy_uj / AMD energy hwmon, joule-delta → watts, counter-wrap safe
│   └── fan.rs     # Linux only: hwmon fanN_input
├── stress.rs      # built-in stress: N threads spinning sqrt+black_box; graceful stop via AtomicBool;
│                  # external `stress`/`stress-ng` passthrough when found on PATH (unix)
├── export.rs      # CSV append logging (--csv), JSON/terminal one-shot formatting
└── config.rs      # load/save TOML: refresh rate, temp threshold, per-sensor visibility
```

Data flow: tick every `--refresh-rate` seconds (default 2.0) → each available `Source::read()` →
append readings to history → render. One-shot modes read once, print, exit — no TUI.

## Feature scope (v1)

**In:** temp/freq/util/power/fan monitoring; graphs with 1000-sample history (one aggregate
series per sensor — avg util / max temp; per-core values shown in the summary sidebar);
temp-threshold color alerts; sensor show/hide toggles (number keys); built-in stress;
external stress/stress-ng passthrough; monitor/stress mode toggle; CSV logging; `--json` and
`--terminal` one-shot output; `--refresh-rate`, `--t-thresh`, `--version` flags;
config persistence; clean shutdown on q/Ctrl-C/SIGTERM that kills stress workers/children first.

**Deferred (YAGNI for v1, documented here so they aren't rediscovered):** MSR throttle-reason
decoding (root + /dev/cpu/N/msr), hooks.d threshold scripts, power-profile/governor write menu,
mouse support, smooth-Unicode graph mode.

## Platform support

- **Linux:** full feature set (power, fans, throttle counts are Linux-only).
- **macOS:** util/freq/temps via sysinfo where the OS exposes them; no power/fans.
- **Windows:** util/freq via sysinfo; temps usually unavailable without vendor drivers.

Every source reports `available()`; unavailable sources are simply not shown. A sensor that
starts failing mid-run keeps its last data and is flagged, never panics (matches s-tui behavior).

## Error handling

- Per-source reads return `Result`; errors mark the source unavailable for that tick, log once
  on state transition (not every tick), and preserve history.
- Stress shutdown: AtomicBool stop flag → join threads; external stress spawned in its own
  process group (unix), killed with the group on exit.
- Terminal restore guaranteed via drop guard even on panic.

## Testing

- Unit tests: RAPL joule-delta/wraparound math, CSV/JSON formatting, config round-trip,
  sysfs parsers against fixture directory trees (no root, no real hardware needed).
- CI: `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check` on ubuntu/macos/windows.

## Repository & releases

- Public GitHub repo `s-tui-rs`, **GPL-2.0 license** — upstream s-tui is GPL-2.0 and this is a
  functional rewrite of it, so matching the license is the safe, respectful choice.
- `.github/workflows/ci.yml`: test matrix on push/PR.
- `.github/workflows/release.yml`: on tag `v*`, build three artifacts and attach to a GitHub
  Release:
  - `x86_64-unknown-linux-musl` (static)
  - `aarch64-apple-darwin`
  - `x86_64-pc-windows-msvc`
- Ship `v0.1.0` at the end of implementation.

## Implementation plan (Hermes task decomposition)

1. **Scaffold** — cargo project, clap CLI skeleton, config module, CI workflow. No deps between later tasks and each other except as noted.
2. **Sources** — Source trait + temp/freq/util/power/fan impls with unit tests (depends on 1).
3. **Stress** — built-in threads + external passthrough (depends on 1).
4. **Export** — CSV/JSON/terminal modes wired to sources (depends on 2).
5. **TUI** — ratatui app: graphs, sidebar, menus, key handling (depends on 2, 3).
6. **Release** — README, release workflow, integration pass, tag v0.1.0 (depends on 4, 5).
