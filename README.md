# 🦀 redisrs — A Redis-Compatible In-Memory Store in Rust

A from-scratch, Redis-compatible in-memory data store built in Rust, powered by the Tokio async runtime. Implements the RESP (Redis Serialization Protocol) and supports core Redis commands, multiple data structures, key expiry, and optional persistence — all with Rust's performance and safety guarantees.

---

## ✨ Features

- ⚡ **Async, multi-client TCP server** via Tokio
- 📦 **Full RESP protocol** parser and serializer
- 🗄️ **Core data structures** — Strings, Lists, Hashes, Sets
- ⏱️ **Key expiry** — `EXPIRE`, `TTL`, `PERSIST`, lazy + active eviction
- 💾 **Persistence** — RDB snapshots and AOF (Append-Only File)
- 📡 **Pub/Sub** — `SUBSCRIBE`, `PUBLISH`, `UNSUBSCRIBE`
- 🧪 **Compatible with `redis-cli`** and any standard Redis client

---

## 🚀 Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.75+ recommended)
- `cargo` (included with Rust)

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

### Test with redis-cli

```bash
# Ping the server
redis-cli -p 6379 ping

# Set and get a key
redis-cli -p 6379 set hello world
redis-cli -p 6379 get hello

# Set a key with expiry
redis-cli -p 6379 set session abc EX 60
redis-cli -p 6379 ttl session
```

---

## 📋 Supported Commands

### Strings
| Command | Syntax | Description |
|---|---|---|
| `PING` | `PING` | Health check, returns PONG |
| `SET` | `SET key value [EX seconds] [PX ms]` | Set a string value with optional expiry |
| `GET` | `GET key` | Get the value of a key |
| `DEL` | `DEL key [key ...]` | Delete one or more keys |
| `EXISTS` | `EXISTS key` | Check if a key exists |
| `KEYS` | `KEYS pattern` | List all keys matching a pattern |

### Expiry
| Command | Syntax | Description |
|---|---|---|
| `EXPIRE` | `EXPIRE key seconds` | Set a timeout on a key |
| `TTL` | `TTL key` | Get remaining time to live |
| `PERSIST` | `PERSIST key` | Remove the expiry from a key |

### Lists
| Command | Syntax | Description |
|---|---|---|
| `LPUSH` | `LPUSH key value [value ...]` | Push values to the left of a list |
| `RPUSH` | `RPUSH key value [value ...]` | Push values to the right of a list |
| `LPOP` | `LPOP key` | Remove and return the leftmost element |
| `RPOP` | `RPOP key` | Remove and return the rightmost element |
| `LRANGE` | `LRANGE key start stop` | Get a range of elements from a list |

### Hashes
| Command | Syntax | Description |
|---|---|---|
| `HSET` | `HSET key field value` | Set a field in a hash |
| `HGET` | `HGET key field` | Get the value of a hash field |
| `HDEL` | `HDEL key field [field ...]` | Delete one or more hash fields |
| `HGETALL` | `HGETALL key` | Get all fields and values in a hash |

### Sets
| Command | Syntax | Description |
|---|---|---|
| `SADD` | `SADD key member [member ...]` | Add members to a set |
| `SMEMBERS` | `SMEMBERS key` | Get all members of a set |
| `SISMEMBER` | `SISMEMBER key member` | Check if a value is in a set |
| `SREM` | `SREM key member [member ...]` | Remove members from a set |

### Pub/Sub
| Command | Syntax | Description |
|---|---|---|
| `SUBSCRIBE` | `SUBSCRIBE channel [channel ...]` | Subscribe to channels |
| `PUBLISH` | `PUBLISH channel message` | Publish a message to a channel |
| `UNSUBSCRIBE` | `UNSUBSCRIBE [channel ...]` | Unsubscribe from channels |

---

## 🏗️ Project Structure

```
redisrs/
├── Cargo.toml
├── Cargo.lock
├── README.md
└── src/
    ├── main.rs           # Entry point, TCP listener setup
    ├── server.rs         # Per-client connection handler
    ├── resp.rs           # RESP protocol parser & serializer
    ├── store.rs          # In-memory store + expiry eviction logic
    ├── commands.rs       # Command dispatch and execution
    └── persistence.rs    # RDB snapshots and AOF logging
```

---

## 🔧 Configuration

Configuration is handled via environment variables or a config file (coming soon):

| Variable | Default | Description |
|---|---|---|
| `REDISRS_PORT` | `6379` | Port to listen on |
| `REDISRS_PERSISTENCE` | `none` | Persistence mode: `none`, `rdb`, `aof` |
| `REDISRS_SNAPSHOT_INTERVAL` | `300` | RDB snapshot interval in seconds |
| `REDISRS_AOF_PATH` | `./appendonly.aof` | Path for AOF file |

