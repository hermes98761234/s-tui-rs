# s-tui-rs

Terminal CPU monitoring and stress testing UI — a Rust rewrite of
[amanusk/s-tui](https://github.com/amanusk/s-tui).

## Features

- **Real-time monitors**: CPU utilisation (per-core + average), frequency (per-core
  with throttle counts on Linux), temperature, and on Linux: power draw and fan RPM.
- **Stress testing**: built-in CPU burn loop (sqrt workload), or launch external
  `stress-ng`/`stress` automatically.
- **Export modes**: one-shot JSON or terminal output, continuous CSV logging.
- **Configurable**: refresh rate, temperature alert threshold, hide/show panes —
  saved as TOML config.
- **TUI key bindings**:
  - `q` — quit
  - `h` — toggle help overlay
  - `s` — toggle built-in stress
  - `e` — toggle external stress (stress-ng/stress)
  - `1`–`9` — toggle visibility of pane N

## Install

### Pre-built binaries

Download a binary from the
[releases page](https://github.com/hermes98761234/s-tui-rs/releases):

| Platform | Binary |
|---|---|
| Linux (x86_64, musl) | `s-tui-rs-x86_64-unknown-linux-musl` |
| macOS (ARM64) | `s-tui-rs-aarch64-apple-darwin` |
| Windows (x86_64) | `s-tui-rs-x86_64-pc-windows-msvc.exe` |

Make the binary executable (`chmod +x`) and place it somewhere on your `PATH`.

### From source

```bash
cargo install --git https://github.com/hermes98761234/s-tui-rs
```

Requires Rust 1.70+.

## Usage

```
Terminal CPU monitoring and stress testing UI

Usage: s-tui-rs [OPTIONS]

Options:
  -j, --json                         Print one reading as JSON and exit
  -t, --terminal                     Print one reading as plain text and exit
  -c, --csv                          Append readings to a CSV file while running
      --csv-file <CSV_FILE>          CSV file path [default: s-tui.csv]
  -r, --refresh-rate <REFRESH_RATE>  Refresh rate in seconds (default 2.0, or saved config value)
      --t-thresh <T_THRESH>          High temperature alert threshold in °C (default 80, or saved config value)
  -h, --help                         Print help
  -V, --version                      Print version
```

### Examples

```bash
# One-shot terminal read
s-tui-rs --terminal

# Continuous CSV logging
s-tui-rs --csv --csv-file readings.csv

# One-shot JSON (for piping to jq)
s-tui-rs --json | jq .

# Interactive TUI with 5-second refresh
s-tui-rs --refresh-rate 5
```

### Terminal one-shot output

```
Util: [Avg 22.9%, Core 0 23.8%, Core 1 20.0%, Core 2 5.0%, ...]
Freq: [Core 0 2016.0MHz, Core 1 2016.0MHz, ...]
Temp: [little_core_thermal temp1 58.2°C, soc_thermal temp1 55.5°C, ...]
```

## Configuration

Config file location (per-platform):

| Platform | Path |
|---|---|
| Linux | `~/.config/s-tui-rs/config.toml` |
| macOS | `~/Library/Application Support/s-tui-rs/config.toml` |
| Windows | `C:\Users\<user>\AppData\Roaming\s-tui-rs\config.toml` |

TOML fields and defaults:

```toml
refresh_rate = 2.0          # Seconds between sensor reads
temp_threshold = 80.0       # High temp alert threshold in °C
hidden_sources = []         # Source pane names to hide (e.g. ["Fan", "Power"])
```

## Platform support

| Sensor | Linux | macOS | Windows |
|---|---|---|---|
| CPU utilisation | ✓ | ✓ | ✓ |
| CPU frequency | ✓ | ✓ | ✓ |
| Temperature | ✓ | Partial | Partial |
| Power draw | ✓ | — | — |
| Fan speed | ✓ | — | — |
| Stress (built-in) | ✓ | ✓ | ✓ |
| Stress (external) | ✓ | — | — |

## License

GNU General Public License v2.0. See [LICENSE](LICENSE).
