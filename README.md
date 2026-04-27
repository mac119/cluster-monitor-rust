# cluster-monitor-rust

A high-performance, real-time Redis traffic capture tool written in Rust. It leverages the Redis `MONITOR` command to intercept and stream every command processed by a Redis cluster — across all proxy instances simultaneously — making it ideal for **auditing**, **monitoring**, **debugging**, and **traffic analysis**.

## Features

- **Real-time traffic capture** — Streams every Redis command as it is executed, with zero delay
- **Cluster-wide coverage** — Connects to all proxy instances concurrently and aggregates their traffic into a single output stream
- **Two discovery modes** — Auto-discover proxies via a VIP address, or specify them explicitly via a config file
- **Multi-threaded** — Each proxy connection runs in its own thread for maximum throughput
- **Static binary** — Compiled as a fully static Linux binary (musl), no runtime dependencies required
- **Lightweight** — Minimal dependencies, small binary size, low resource usage

## Use Cases

| Scenario | Description |
|----------|-------------|
| **Security Auditing** | Record all commands and keys accessed to detect unauthorized access or suspicious patterns |
| **Performance Monitoring** | Identify hot keys, slow commands, and traffic spikes in real time |
| **Debugging** | Trace exactly what commands an application is sending to Redis |
| **Traffic Analysis** | Analyze command distribution, key namespaces, and access frequency |
| **Compliance** | Maintain an audit trail of all Redis operations for regulatory requirements |

## How It Works

```
                    ┌─────────────────────────────────────────┐
                    │            Redis Cluster                │
                    │                                         │
  VIP / proxyInfo ──►  Proxy-1  Proxy-2  Proxy-3  ...        │
                    │     │        │        │                 │
                    │  MONITOR  MONITOR  MONITOR              │
                    └─────┼────────┼────────┼─────────────────┘
                          │        │        │
                          └────────┴────────┘
                                   │
                             stdout stream
                          (all commands merged)
```

1. Connect to the cluster via VIP or proxy list
2. Issue `MONITOR` to each proxy instance in parallel
3. All intercepted commands are printed to stdout in real time
4. Pipe or redirect the output for logging, alerting, or analysis

## Building

### Build for the current platform

```bash
sh build.sh
```

Or with Cargo directly:

```bash
cargo build --release
mkdir -p bin
cp target/release/monitor-on-vip target/release/monitor-on-proxy bin/
```

### Cross-compile for Linux x86_64 (from macOS)

```bash
# Add the musl target (one-time setup)
rustup target add x86_64-unknown-linux-musl

# Install the cross-linker (macOS only)
brew install filosottile/musl-cross/musl-cross

# Build
cargo build --release --target x86_64-unknown-linux-musl
```

The resulting binaries in `target/x86_64-unknown-linux-musl/release/` are fully static ELF executables that run on any Linux x86_64 system without additional dependencies.

## Usage

### Mode 1 — VIP Mode (recommended)

Automatically discovers all proxy instances behind a VIP by repeatedly connecting and reading the `proxyId` field, then monitors all of them concurrently.

```bash
./bin/monitor-on-vip --host <VIP> --port <PORT> --passwd <PASSWORD>
```

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--host` | `-H` | `127.0.0.1` | VIP address of the Redis cluster |
| `--port` | `-p` | `6379` | Port |
| `--passwd` | `-a` | *(required)* | Redis AUTH password |

**Example:**

```bash
./bin/monitor-on-vip --host 10.0.0.1 --port 6379 --passwd mysecret
```

### Mode 2 — Proxy Direct Mode

Connect directly to a known list of proxy instances specified in a `proxyInfo` file.

**`proxyInfo` format:**

```
<password>
<host1> <port1>
<host2> <port2>
...
```

**Example `proxyInfo`:**

```
mysecret
10.0.0.11 6379
10.0.0.12 6379
10.0.0.13 6379
```

**Run:**

```bash
sh monitor.sh
```

Or directly:

```bash
cp proxyInfo bin/
./bin/monitor-on-proxy
```

## Output

Each line printed to stdout represents one Redis command intercepted from the cluster:

```
1745123456.123456 [0 10.0.0.5:41234] "SET" "user:1001:session" "abc123"
1745123456.124001 [0 10.0.0.6:38821] "GET" "user:1001:profile"
1745123456.125312 [0 10.0.0.5:41234] "EXPIRE" "user:1001:session" "3600"
```

You can pipe the output to standard Unix tools for real-time filtering:

```bash
# Watch only SET commands
./bin/monitor-on-vip -H 10.0.0.1 -p 6379 -a secret | grep '"SET"'

# Log to file with timestamps
./bin/monitor-on-vip -H 10.0.0.1 -p 6379 -a secret >> /var/log/redis-audit.log

# Count commands per second
./bin/monitor-on-vip -H 10.0.0.1 -p 6379 -a secret | pv -l -r > /dev/null
```

## Project Structure

```
.
├── src/
│   ├── monitor_on_vip.rs       # VIP mode: auto-discover proxies via VIP
│   └── monitor_on_proxy.rs     # Direct mode: connect to proxies from proxyInfo
├── proxyInfo                   # Proxy list config file (direct mode)
├── build.sh                    # Build script
├── monitor.sh                  # Run script for direct mode
└── Cargo.toml
```

## Requirements

- Rust 1.65+ (for building)
- Redis cluster with `MONITOR` command enabled on proxy instances
- Network access to the VIP or proxy addresses

## License

MIT
