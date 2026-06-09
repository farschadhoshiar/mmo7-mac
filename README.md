# mmo7-mac

Terminal UI configurator for the **Mad Catz M.M.O. 7+** wireless gaming mouse on macOS.

Mad Catz only ships Windows drivers for the M.M.O. 7+ (USB `0738:1C02`). This project aims to fill that gap with an open-source, Rust-based tool that runs in your terminal.

## Status

🚧 **Phase 0 — HID sniffer.** The project currently opens the mouse via HIDAPI and displays raw input reports in a [ratatui](https://ratatui.rs) interface. This is the groundwork for reverse-engineering the report format so that buttons, modes, DPI and RGB can be mapped in later phases.

### Roadmap

| Phase | Goal | Status |
|---|---|---|
| 0 | Open device, render raw HID reports live in TUI | done |
| 1 | Interactive wizard: prompt user per button, diff baseline vs press, emit `mmo7-mapping.toml` | done |
| 2 | Map buttons → key sequences via TOML config + CGEvent | planned |
| 3 | Reverse-engineer output reports for DPI / RGB / on-board profiles | later |

## Requirements

- macOS (Apple Silicon or Intel)
- Rust 1.85+ (2024 edition)
- Mad Catz M.M.O. 7+ mouse (`0738:1C02`)

## Build & run

```sh
cargo run --release
```

On first launch, macOS may prompt for **Input Monitoring** permission — grant it in *System Settings → Privacy & Security → Input Monitoring*.

### Why your cursor still works

hidapi on macOS opens HID devices with `kIOHIDOptionsTypeSeizeDevice`, meaning the OS hands over exclusive ownership. To avoid hijacking your pointer/keystrokes, the sniffer **skips the `Generic Desktop / Mouse` and `Generic Desktop / Keyboard` top-level collections** and only opens vendor, consumer, and other auxiliary interfaces. Skipped interfaces appear as `○` in the sidebar.

## Hardware identification

```sh
ioreg -p IOUSB -l | grep -B 1 'M.M.O. 7'
```

Expected vendor ID `0x0738` (Mad Catz), product ID `0x1C02`.

## Controls

Two views, switch with `Tab`.

### Wizard (default)

| Key | Action |
|---|---|
| `Space` | begin baseline / start recording the current probe |
| `Enter` | accept the detected mapping, advance to next probe |
| `r` | retry the current probe |
| `n` | skip the current probe |
| `b` | re-baseline (e.g. if you accidentally bumped the mouse) |

When the wizard finishes, the discovered mapping is written to `mmo7-mapping.toml` in the current directory.

### Sniffer (advanced)

| Key | Action |
|---|---|
| `p` / `Space` | pause / resume capture |
| `c` | clear report buffer |
| `↑↓` / `jk` | scroll |
| `g` / `G` | jump to top / bottom |
| `1`–`9` | toggle visibility of an interface |

### Always

| Key | Action |
|---|---|
| `Tab` | toggle view |
| `q` / `Esc` / `Ctrl-C` | quit |

## License

MIT — see [LICENSE](LICENSE).
