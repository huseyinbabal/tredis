
<img width="693" height="336" alt="Screenshot 2026-01-09 at 17 10 47" src="https://github.com/user-attachments/assets/76b38b31-21d9-4b1e-a10c-be2942e86626" />

# tredis - Terminal UI for Redis

**tredis** provides a terminal UI to interact with your Redis servers. The aim of this project is to make it easier to navigate, observe, and manage your Redis data in the wild.

---

[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

---

## Showcase

<div align="center">
  <video src="https://github.com/user-attachments/assets/52bad0e2-873c-4389-9f89-523383981a4e" 
         controls 
         autoplay 
         loop 
         muted
         width="600">
    Tarayıcınız video etiketini desteklemiyor.
  </video>
</div>

---

## Features

- **Multi-Server Support** - Manage multiple Redis servers from a single interface
- **TLS Support** - Connect to Redis servers with TLS encryption (Upstash, Redis Cloud, etc.)
- **Key Browser** - Browse and search keys with pagination
- **Data Type Support** - View and inspect String, List, Set, Hash, ZSet, and Stream data types
- **Real-time Monitoring** - Monitor Redis commands in real-time
- **Pub/Sub** - Subscribe to channels and view messages
- **Streams** - Browse and consume Redis Streams
- **Client List** - View connected clients
- **Slowlog** - Inspect slow queries
- **Server Info** - View detailed server information with vim-style search (`/`, `n`, `N`)
- **ACL Management** - View ACL users and permissions
- **Configuration** - Browse and view Redis configuration
- **Keyboard-Driven** - Vim-like navigation and commands
- **Filtering** - Filter keys by pattern

---

## Installation

### Homebrew (macOS/Linux)

```bash
brew install huseyinbabal/tap/tredis
```

### Scoop (Windows)

```powershell
scoop bucket add huseyinbabal https://github.com/huseyinbabal/scoop-bucket
scoop install tredis
```

### Download Pre-built Binaries

Download the latest release from the [Releases page](https://github.com/huseyinbabal/tredis/releases/latest).

| Platform | Architecture | Download |
|----------|--------------|----------|
| **macOS** | Apple Silicon (M1/M2/M3) | `tredis-aarch64-apple-darwin.tar.gz` |
| **macOS** | Intel | `tredis-x86_64-apple-darwin.tar.gz` |
| **Linux** | x86_64 | `tredis-x86_64-unknown-linux-gnu.tar.gz` |
| **Linux** | ARM64 | `tredis-aarch64-unknown-linux-gnu.tar.gz` |
| **Windows** | x86_64 | `tredis-x86_64-pc-windows-msvc.zip` |

#### Quick Install (macOS/Linux)

```bash
# macOS Apple Silicon
curl -sL https://github.com/huseyinbabal/tredis/releases/latest/download/tredis-aarch64-apple-darwin.tar.gz | tar xz
sudo mv tredis /usr/local/bin/

# macOS Intel
curl -sL https://github.com/huseyinbabal/tredis/releases/latest/download/tredis-x86_64-apple-darwin.tar.gz | tar xz
sudo mv tredis /usr/local/bin/

# Linux x86_64
curl -sL https://github.com/huseyinbabal/tredis/releases/latest/download/tredis-x86_64-unknown-linux-gnu.tar.gz | tar xz
sudo mv tredis /usr/local/bin/

# Linux ARM64
curl -sL https://github.com/huseyinbabal/tredis/releases/latest/download/tredis-aarch64-unknown-linux-gnu.tar.gz | tar xz
sudo mv tredis /usr/local/bin/
```

#### Windows

1. Download `tredis-x86_64-pc-windows-msvc.zip` from the [Releases page](https://github.com/huseyinbabal/tredis/releases/latest)
2. Extract the zip file
3. Add the extracted folder to your PATH, or move `tredis.exe` to a directory in your PATH

### Using Cargo

```bash
cargo install tredis
```

### From Source

tredis is built with Rust. Make sure you have Rust 1.70+ installed, along with a C compiler and linker.

#### Build Dependencies

| Platform | Install Command |
|----------|-----------------|
| **Amazon Linux / RHEL / Fedora** | `sudo yum groupinstall "Development Tools" -y` |
| **Ubuntu / Debian** | `sudo apt update && sudo apt install build-essential -y` |
| **macOS** | `xcode-select --install` |
| **Windows** | Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) |

```bash
# Clone the repository
git clone https://github.com/huseyinbabal/tredis.git
cd tredis

# Build and run
cargo build --release
./target/release/tredis
```

---

## Quick Start

```bash
# Launch tredis (will prompt to add a server if none configured)
tredis

# Launch with a specific host and port
tredis --host localhost --port 6379

# Connect to a specific database
tredis --host localhost --port 6379 --db 1

# Enable debug logging
tredis --log-level debug
```

### Adding a Server

When you first launch tredis, you'll be prompted to add a server. Enter:
- **Name**: A friendly name for the server (e.g., "production", "local")
- **URI**: The Redis connection URI

#### URI Format

```
redis://[[user]:password@]host[:port][/db]
rediss://[[user]:password@]host[:port][/db]  # TLS
```

**Examples:**
```
redis://localhost:6379                    # Local Redis
redis://localhost:6379/1                  # Local Redis, database 1
redis://:mypassword@localhost:6379        # With password
rediss://default:token@my.upstash.io:6379 # Upstash (TLS)
rediss://user:pass@redis.cloud.com:6380   # Redis Cloud (TLS)
```

### Log File Locations

| Platform | Path |
|----------|------|
| **Linux** | `~/.config/tredis/tredis.log` |
| **macOS** | `~/.config/tredis/tredis.log` |
| **Windows** | `%APPDATA%\tredis\tredis.log` |

### Configuration File

Server configurations are stored in:

| Platform | Path |
|----------|------|
| **Linux** | `~/.config/tredis/config.yaml` |
| **macOS** | `~/.config/tredis/config.yaml` |
| **Windows** | `%APPDATA%\tredis\config.yaml` |

---

## Key Bindings

| Action | Key | Description |
|--------|-----|-------------|
| **Navigation** | | |
| Move up | `k` / `↑` | Move selection up |
| Move down | `j` / `↓` | Move selection down |
| Top | `gg` | Jump to first item |
| Bottom | `G` | Jump to last item |
| **Pagination** | | |
| Next page | `]` | Load next page of results |
| Previous page | `[` | Load previous page of results |
| **Views** | | |
| Resources | `:` | Open resource selector |
| Describe | `Enter` / `d` | View key/resource details |
| Back | `Esc` / `Backspace` | Go back to previous view |
| **Actions** | | |
| Refresh | `R` | Refresh current view |
| Filter | `/` | Filter keys (in Keys view) |
| Connect | `c` | Connect to selected server |
| Add server | `a` | Add a new server |
| Delete | `Ctrl-d` | Delete selected key/server |
| Quit | `Ctrl-c` / `q` | Exit tredis |
| **Info Search** | | |
| Search | `/` | Start search in Info view |
| Next match | `n` | Jump to next match |
| Previous match | `N` | Jump to previous match |
| Clear search | `Esc` | Clear search and highlights |
| **Streams** | | |
| Consume | `c` | Start consuming stream messages |
| Stop | `Esc` | Stop consuming |
| **PubSub** | | |
| Test Subscribe | `s` | Subscribe to a channel |
| Stop | `Esc` | Stop subscription |
| **Monitor** | | |
| Clear | `R` | Clear monitor entries |

---

## Resource Navigation

Press `:` to open the resource picker. Available resources:

| Resource | Description |
|----------|-------------|
| `keys` | Browse Redis keys |
| `servers` | Manage server connections |
| `clients` | View connected clients |
| `info` | Server information |
| `slowlog` | Slow query log |
| `config` | Redis configuration |
| `acl` | ACL users |
| `monitor` | Real-time command monitor |
| `streams` | Redis Streams |
| `pubsub` | Pub/Sub channels |

---

## Supported Data Types

tredis supports viewing all Redis data types:

| Type | View Support |
|------|--------------|
| **String** | Full value display |
| **List** | All elements with index |
| **Set** | All members |
| **Hash** | All field-value pairs |
| **Sorted Set** | Members with scores |
| **Stream** | Messages with IDs and fields |

---

## Cloud Redis Support

tredis supports connecting to cloud Redis providers:

| Provider | URI Format |
|----------|------------|
| **Upstash** | `rediss://default:<token>@<endpoint>.upstash.io:6379` |
| **Redis Cloud** | `rediss://default:<password>@<endpoint>.redis.cloud.redislabs.com:<port>` |
| **AWS ElastiCache** | `redis://<endpoint>.cache.amazonaws.com:6379` |
| **Azure Cache** | `rediss://:<key>@<name>.redis.cache.windows.net:6380` |

> **Note:** Cloud providers typically require TLS (`rediss://`). Check your provider's documentation for the exact connection string format.

---

## Known Issues

- Some Redis commands may not be available on all Redis versions
- Cluster mode is detected but individual node management is not yet supported
- Large keys may take time to load in the describe view

---

## Contributing

Contributions are welcome! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

---

## Acknowledgments

- Inspired by [k9s](https://github.com/derailed/k9s) - the awesome Kubernetes CLI
- Inspired by [taws](https://github.com/huseyinbabal/taws) - Terminal UI for AWS
- Built with [Ratatui](https://github.com/ratatui-org/ratatui) - Rust TUI library

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

<p align="center">
  Made with ❤️ for the Redis community
</p>
