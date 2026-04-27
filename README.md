# Redis Cluster Monitor (Rust)

Redis Proxy 集群实时监控工具的 Rust 实现。

## 功能

通过 Redis 的 `MONITOR` 命令，实时捕获集群中所有 Proxy 实例处理的命令流。

提供两种监控模式：
- **VIP 模式**：通过 VIP 地址自动发现所有后端 Proxy 实例
- **Proxy 直连模式**：通过配置文件直接指定 Proxy 地址列表

## 编译

```bash
sh build.sh
```

或者直接使用 cargo：

```bash
cargo build --release
```

## 使用方法

### 模式一：VIP 模式（推荐）

```bash
./bin/monitor-on-vip --host 127.0.0.1 --port 6379 --passwd 12345567
```

参数说明：
- `--host` / `-H`：VIP 地址（默认 `127.0.0.1`）
- `--port` / `-p`：端口（默认 `6379`）
- `--passwd` / `-a`：Redis 认证密码（必填）

### 模式二：Proxy 直连模式

1. 编辑 `proxyInfo` 文件，格式如下：

```
<password>
<host1> <port1>
<host2> <port2>
...
```

2. 运行：

```bash
sh monitor.sh
```
