# Hayaku-Ike

**Hayaku-Ike** — a Linux speed booster daemon and tray app. It dynamically monitors system load and runs performance-boosting tasks when the system is idle.

## Features

* Automatic CPU performance mode switching
* Swappiness tuning and page cache clearing
* Swap refresh if used
* Tray icon with dynamic states: Idle, Busy, Paused
* Desktop notifications with icon support
* Configurable thresholds and intervals via `config.toml`

## Installation

1. Build the project:

```bash
cargo build --release
```

2. Install the systemd service (requires root):

```bash
sudo ./target/release/hayaku-ike --install-service
```

This copies `assets/service/hayaku-ike.service` to `/etc/systemd/system/`, enables, and starts it.

## Configuration

Default configuration is in `config.toml`:

```toml
idle_load_threshold = 0.2
min_interval = 30
max_interval = 300
load_change_threshold = 0.05
min_idle_cycles_for_notify = 2

[icons]
active = "assets/icons/player-play.svg"
busy = "assets/icons/cpu.svg"
paused = "assets/icons/player-stop.svg"
```

You can tweak thresholds, intervals, and icons to suit your workflow.

## Usage

Run directly for testing:

```bash
cargo run --release
```

Tray icon allows:

* Pause/Resume booster
* Quit application

Notifications show when state changes or booster runs.

## Logging

Logs are written to:

```
/var/log/hayaku-ike.log
```

## License

MIT — see LICENSE file.
