# rstop
A real-time system monitor for Linux with zram statistics.

<img width="351" height="357" alt="image" src="https://github.com/user-attachments/assets/0ae7c081-414e-4c1b-8953-bc7a160f4e74" />

## Features

- CPU, RAM, and SWAP usage monitoring
- Load average (1m, 5m, 10m)
- ZRAM compression ratio statistics
- Health score based on system performance
- Renice mode to kill top CPU-consuming processes with selection (no more typing PIDs!)
- Config file with options to hide based on minimum CPU/Memory usage to prevent clutter

## Install

### Ubuntu 22.04 LTS

```bash
curl -sL https://github.com/dpriskorn/rstop/releases/download/v0.1.0/install.sh | bash
```

Then add to PATH:
```bash
export PATH=$HOME/.local/rstop/bin:$PATH
```

### From Source

```bash
cargo build --release
```

The binary will be at `target/release/rstop`.

## Usage

```bash
./target/release/rstop          # Default 2s interval
./target/release/rstop -i 1     # 1s interval
./target/release/rstop --interval 0.5  # 0.5s interval
```

## License

GPLv3 or later