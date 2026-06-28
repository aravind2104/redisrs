# 🦀 redisrs — A Redis-Compatible In-Memory Store in Rust

A from-scratch, Redis-compatible in-memory data store built in Rust, powered by the Tokio async runtime. Implements the full RESP (Redis Serialization Protocol), supports multiple data structures, key expiry, persistence, Pub/Sub messaging, and password authentication — all with Rust's performance and safety guarantees.

---

## ✨ Features

- ⚡ **Async, multi-client TCP server** — concurrent connections via Tokio tasks
- 📦 **Full RESP protocol** — complete parser and serializer for all 5 data types
- 🗄️ **Multiple data structures** — Strings, Lists, Hashes, Sets
- ⏱️ **Key expiry** — lazy eviction on access + active background eviction every second
- 💾 **Dual persistence** — RDB snapshots and AOF (Append-Only File) logging
- 📡 **Pub/Sub** — channel-based messaging with `SUBSCRIBE`, `PUBLISH`, `UNSUBSCRIBE`
- 🔐 **Password authentication** — per-connection `AUTH` with `config.toml` support
- ⚙️ **Config file** — port, password, snapshot interval, AOF path all configurable
- 🧪 **Compatible with `redis-cli`** and any standard Redis client

---

## 🚀 Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.75+ recommended)
- `cargo` (included with Rust)
- `redis-cli` (optional, for testing) — install via `sudo apt install redis-tools` or `brew install redis`

### Build & Run

```bash
# Clone the repository
git clone https://github.com/your-username/redisrs.git
cd redisrs

# Build the project
cargo build --release

# Run the server (default port: 6379)
cargo run --release
```

### Configuration

Create a `config.toml` in the project root to configure the server:

```toml
port = 6379
password = "your_password"
snapshot_interval = 300
aof_path = "appendonly.aof"
```

All fields are optional — the server starts with sensible defaults if `config.toml` is missing.

### Test with redis-cli

```bash
# Without authentication
redis-cli -p 6379 ping
redis-cli -p 6379 set hello world
redis-cli -p 6379 get hello

# With authentication
redis-cli -p 6379 -a your_password get hello

# Interactive mode
redis-cli -p 6379
127.0.0.1:6379> auth your_password
127.0.0.1:6379> set hello world
127.0.0.1:6379> get hello
```

---

## 📋 Supported Commands

### General
| Command | Syntax | Description |
|---|---|---|
| `PING` | `PING [message]` | Health check, returns PONG or echoes message |
| `AUTH` | `AUTH password` | Authenticate with the server |

### Strings
| Command | Syntax | Description |
|---|---|---|
| `SET` | `SET key value [EX seconds] [PX ms]` | Set a string value with optional expiry |
| `GET` | `GET key` | Get the value of a key |
| `DEL` | `DEL key [key ...]` | Delete one or more keys, returns count |
| `EXISTS` | `EXISTS key [key ...]` | Returns count of existing keys |
| `KEYS` | `KEYS *` | List all keys |

### Expiry
| Command | Syntax | Description |
|---|---|---|
| `EXPIRE` | `EXPIRE key seconds` | Set a timeout on a key |
| `TTL` | `TTL key` | Returns remaining TTL in seconds, -1 if no expiry, -2 if missing |
| `PERSIST` | `PERSIST key` | Remove the expiry from a key |

### Lists
| Command | Syntax | Description |
|---|---|---|
| `LPUSH` | `LPUSH key value [value ...]` | Push values to the left of a list |
| `RPUSH` | `RPUSH key value [value ...]` | Push values to the right of a list |
| `LPOP` | `LPOP key` | Remove and return the leftmost element |
| `RPOP` | `RPOP key` | Remove and return the rightmost element |
| `LRANGE` | `LRANGE key start stop` | Get a range of elements (supports negative indices) |

### Hashes
| Command | Syntax | Description |
|---|---|---|
| `HSET` | `HSET key field value` | Set a field in a hash, returns 1 if new, 0 if updated |
| `HGET` | `HGET key field` | Get the value of a hash field |
| `HDEL` | `HDEL key field [field ...]` | Delete one or more hash fields, returns count |
| `HGETALL` | `HGETALL key` | Get all fields and values as a flat array |

### Sets
| Command | Syntax | Description |
|---|---|---|
| `SADD` | `SADD key member [member ...]` | Add members to a set, returns count of newly added |
| `SMEMBERS` | `SMEMBERS key` | Get all members of a set |
| `SISMEMBER` | `SISMEMBER key member` | Returns 1 if member exists, 0 otherwise |
| `SREM` | `SREM key member [member ...]` | Remove members from a set, returns count removed |

### Pub/Sub
| Command | Syntax | Description |
|---|---|---|
| `SUBSCRIBE` | `SUBSCRIBE channel` | Subscribe to a channel and listen for messages |
| `PUBLISH` | `PUBLISH channel message` | Publish a message, returns subscriber count |
| `UNSUBSCRIBE` | `UNSUBSCRIBE channel` | Unsubscribe and return to normal command mode |

---

## 🏗️ Project Structure

```
redisrs/
├── Cargo.toml
├── Cargo.lock
├── config.toml          # Server configuration
├── README.md
├── dump.rdb             # RDB snapshot (auto-generated)
├── appendonly.aof       # AOF log (auto-generated)
└── src/
    ├── main.rs          # Entry point, TCP listener, startup
    ├── config.rs        # Config file parsing and defaults
    ├── server.rs        # Per-client connection handler + auth
    ├── resp.rs          # RESP protocol parser & serializer
    ├── store.rs         # In-memory store, expiry, pub/sub channels
    ├── commands.rs      # Command dispatch and execution
    └── persistence.rs   # RDB snapshots and AOF logging
```

---

## 💾 Persistence

redisrs supports two persistence strategies, chosen automatically on startup:

**RDB (Redis Database Snapshot)**
- Serializes the entire store to `dump.rdb` periodically using `serde` + `bincode`
- Interval is configurable via `snapshot_interval` in `config.toml`

**AOF (Append-Only File)**
- Every write command is appended to `appendonly.aof` in RESP format
- On startup, the file is replayed command by command to restore state

**Startup priority:** If `appendonly.aof` exists it is used (most up to date). Otherwise `dump.rdb` is loaded. If neither exists, the server starts fresh.

---

## ⏱️ Key Expiry

Two strategies run together to handle expired keys:

- **Lazy eviction** — keys are checked on every read (`GET`, `HGET`, etc.) and removed if expired. Guarantees correctness on access.
- **Active eviction** — a background Tokio task sweeps the entire store every second and removes expired keys. Prevents memory buildup from unread keys.

---

## 🔐 Authentication

If `password` is set in `config.toml`, clients must authenticate before sending any commands:

```bash
redis-cli -p 6379 -a your_password ping
```

Or in interactive mode:
```
127.0.0.1:6379> auth your_password
OK
```

Unauthenticated clients receive `NOAUTH Authentication required` for every command except `AUTH`.

---

## 📡 Pub/Sub

Pub/Sub uses `tokio::sync::broadcast` channels for fan-out message delivery. Subscribed clients enter a dedicated listening mode and can only send `UNSUBSCRIBE` to return to normal command mode.

```bash
# Terminal 1 — subscribe
redis-cli -p 6379 subscribe news

# Terminal 2 — publish
redis-cli -p 6379 publish news "hello world"

# Terminal 1 output:
# 1) "message"
# 2) "news"
# 3) "hello world"
```

---

## 🧪 Running Tests

```bash
# Run all unit tests
cargo test

# Run with output
cargo test -- --nocapture
```

The RESP parser has full unit test coverage including simple strings, bulk strings, null types, arrays, incomplete data, and round-trip serialization.

---

## 🛣️ Roadmap

- [x] Async TCP server with Tokio
- [x] Full RESP protocol parser & serializer
- [x] String commands — `SET`, `GET`, `DEL`, `EXISTS`, `KEYS`
- [x] Key expiry — `EXPIRE`, `TTL`, `PERSIST`, lazy + active eviction
- [x] List commands — `LPUSH`, `RPUSH`, `LPOP`, `RPOP`, `LRANGE`
- [x] Hash commands — `HSET`, `HGET`, `HDEL`, `HGETALL`
- [x] Set commands — `SADD`, `SMEMBERS`, `SISMEMBER`, `SREM`
- [x] RDB persistence
- [x] AOF persistence
- [x] Pub/Sub — `SUBSCRIBE`, `PUBLISH`, `UNSUBSCRIBE`
- [x] Password authentication
- [x] Config file support
- [ ] Sorted Sets — `ZADD`, `ZRANGE`, `ZSCORE`
- [ ] `SCAN` cursor-based iteration
- [ ] `MULTI` / `EXEC` transactions
- [ ] `INFO` server stats command
- [ ] Multiple database support — `SELECT`

---

## 🤝 Contributing

Contributions are welcome! Please open an issue first to discuss what you'd like to change. Make sure all tests pass before submitting a pull request.

1. Fork the repository
2. Create a feature branch (`git checkout -b feat/sorted-sets`)
3. Commit your changes (`git commit -m 'Add sorted set support'`)
4. Push to your branch (`git push origin feat/sorted-sets`)
5. Open a Pull Request

---

## 📄 License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.

---

## 🙏 Acknowledgements

- [Redis](https://redis.io/) — the original, for the protocol spec and inspiration
- [Tokio](https://tokio.rs/) — the async runtime powering this server
- [Mini Redis](https://github.com/tokio-rs/mini-redis) — Tokio's official educational Redis clone
- [RESP Protocol Specification](https://redis.io/docs/reference/protocol-spec/)
- [Crafting Interpreters](https://craftinginterpreters.com/) — inspiration for the structured build approach