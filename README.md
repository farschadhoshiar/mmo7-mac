# mmo7-mac

Terminal UI configurator for the **Mad Catz M.M.O. 7+** wireless gaming mouse on macOS.

Mad Catz only ships Windows drivers for the M.M.O. 7+ (USB `0738:1C02`). This project aims to fill that gap with an open-source, Rust-based tool that runs in your terminal.

## Status

🚧 **Phase 0 — HID sniffer.** The project currently opens the mouse via HIDAPI and displays raw input reports in a [ratatui](https://ratatui.rs) interface. This is the groundwork for reverse-engineering the report format so that buttons, modes, DPI and RGB can be mapped in later phases.

### Roadmap

| Phase | Goal | Status |
|---|---|---|
| 0 | Open device, render raw HID reports live in TUI | in progress |
| 1 | Decode button / mode / shift bits from reports | planned |
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

## Hardware identification

```sh
ioreg -p IOUSB -l | grep -B 1 'M.M.O. 7'
```

Expected vendor ID `0x0738` (Mad Catz), product ID `0x1C02`.

## Controls

| Key | Action |
|---|---|
| `q` / `Esc` | quit |
| `↑` / `↓` | scroll report history |
| `c` | clear report history |
| `p` | pause / resume capture |

## License

MIT — see [LICENSE](LICENSE).
