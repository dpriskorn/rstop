# RTOP
A real-time system monitor for Linux with zram statistics.

<img width="351" height="357" alt="image" src="https://github.com/user-attachments/assets/0ae7c081-414e-4c1b-8953-bc7a160f4e74" />

## Features

- CPU, RAM, and SWAP usage monitoring
- Load average (1m, 5m, 10m)
- ZRAM compression ratio statistics
- Health score based on system performance
- Renice mode to kill top CPU-consuming processes with selection (no more typing PIDs!)
- Config file with options to hide based on minimum CPU/Memory usage to prevent clutter

## Controls

| Key | Action |
|-----|--------|
| `q` / `ESC` | Quit |
| `p` | Pause display |
| `a` | Toggle advanced mode |
| `h` | Toggle help |
| `r` | Enter renice mode |

### Renice Mode

| Key | Action |
|-----|--------|
| `Up/Down` | Select process |
| `Left/Right` | Toggle signal (9=kill, 15=term) |
| `Enter` | Send signal |
| `ESC` | Exit renice mode |

## Building

```bash
cargo build --release
```

The binary will be at `target/release/rtop`.

## Usage

```bash
./target/release/rtop          # Default 2s interval
./target/release/rtop -i 1     # 1s interval
./target/release/rtop --interval 0.5  # 0.5s interval
```

## License

GPLv3 or later
