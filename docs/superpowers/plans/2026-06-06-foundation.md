# AirOS Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Vytvořit Cargo workspace se sdílenými IPC protokoly (AirProto + AirBus), které tvoří základ komunikace mezi všemi AirOS komponentami.

**Architecture:** Mono-repo Cargo workspace s oddělennými crates. `airproto` definuje binární IPC protokol mezi aplikacemi a AirComp (compositorem). `airbus` definuje MessagePack IPC protokol mezi aplikacemi a AirShell. Oba crates jsou pure-Rust knihovny bez platformních závislostí — lze je testovat na libovolném OS.

**Tech Stack:** Rust 1.78+, `bincode 2` (binární serializace pro AirProto), `rmp-serde` (MessagePack pro AirBus), `serde`, `tokio` (async Unix sockets), `tempfile` (testy)

---

## Soubory

```
airos/
├── Cargo.toml                          # workspace root
├── crates/
│   ├── airproto/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                  # re-exports
│   │       ├── types.rs                # WindowId, Size, Point, Rect
│   │       ├── messages.rs             # ClientMessage, ServerMessage enums
│   │       └── connection.rs           # AirProtoConn (read/write framed messages)
│   └── airbus/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs                  # re-exports
│           ├── messages.rs             # BusMessage enum
│           └── connection.rs           # AirBusConn (read/write MessagePack)
```

---

## Task 1: Cargo Workspace

**Files:**
- Create: `Cargo.toml`
- Create: `crates/airproto/Cargo.toml`
- Create: `crates/airbus/Cargo.toml`

- [ ] **Step 1: Vytvoř workspace root Cargo.toml**

```toml
# Cargo.toml
[workspace]
resolver = "2"
members = [
    "crates/airproto",
    "crates/airbus",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["AirOS Contributors"]

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
bincode = "2"
rmp-serde = "1"
tokio = { version = "1", features = ["net", "io-util", "macros", "rt-multi-thread"] }
tempfile = "3"
```

- [ ] **Step 2: Vytvoř airproto Cargo.toml**

```toml
# crates/airproto/Cargo.toml
[package]
name = "airproto"
version.workspace = true
edition.workspace = true

[dependencies]
serde = { workspace = true }
bincode = { workspace = true }
tokio = { workspace = true }

[dev-dependencies]
tempfile = { workspace = true }
tokio = { workspace = true, features = ["test-util"] }
```

- [ ] **Step 3: Vytvoř airbus Cargo.toml**

```toml
# crates/airbus/Cargo.toml
[package]
name = "airbus"
version.workspace = true
edition.workspace = true

[dependencies]
serde = { workspace = true }
rmp-serde = { workspace = true }
tokio = { workspace = true }

[dev-dependencies]
tempfile = { workspace = true }
tokio = { workspace = true, features = ["test-util"] }
```

- [ ] **Step 4: Ověř že workspace se kompiluje**

```bash
cargo check
```

Očekávaný výstup: `error[E0601]: main function not found` nebo čisté `Finished` — záleží jestli jsou src/lib.rs soubory. Pokud hlásí "couldn't find Cargo.toml", zkontroluj paths v workspace members.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml crates/airproto/Cargo.toml crates/airbus/Cargo.toml
git commit -m "chore: initialize Cargo workspace with airproto and airbus crates"
```

---

## Task 2: AirProto — typy

**Files:**
- Create: `crates/airproto/src/lib.rs`
- Create: `crates/airproto/src/types.rs`

- [ ] **Step 1: Napiš failing test pro typy**

```rust
// crates/airproto/src/types.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WindowId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_id_equality() {
        assert_eq!(WindowId(1), WindowId(1));
        assert_ne!(WindowId(1), WindowId(2));
    }

    #[test]
    fn size_fields() {
        let s = Size { width: 800, height: 600 };
        assert_eq!(s.width, 800);
        assert_eq!(s.height, 600);
    }

    #[test]
    fn point_fields() {
        let p = Point { x: 1.5, y: 2.5 };
        assert!((p.x - 1.5).abs() < f32::EPSILON);
        assert!((p.y - 2.5).abs() < f32::EPSILON);
    }
}
```

- [ ] **Step 2: Spusť testy — ověř že failing (soubor neexistuje)**

```bash
cargo test -p airproto 2>&1 | head -20
```

Očekávaný výstup: `error[E0583]: file not found for module` nebo podobná chyba kompilace.

- [ ] **Step 3: Vytvoř lib.rs**

```rust
// crates/airproto/src/lib.rs
pub mod types;
pub mod messages;
pub mod connection;

pub use types::{WindowId, Size, Point};
pub use messages::{ClientMessage, ServerMessage};
pub use connection::AirProtoConn;
```

- [ ] **Step 4: Spusť testy pro types**

```bash
cargo test -p airproto types
```

Očekávaný výstup: `test types::tests::window_id_equality ... ok` atd. 3 testy passing.

- [ ] **Step 5: Commit**

```bash
git add crates/airproto/src/
git commit -m "feat(airproto): add WindowId, Size, Point types"
```

---

## Task 3: AirProto — zprávy

**Files:**
- Create: `crates/airproto/src/messages.rs`

- [ ] **Step 1: Napiš failing test pro serializaci zpráv**

```rust
// crates/airproto/src/messages.rs
use serde::{Deserialize, Serialize};
use crate::types::{WindowId, Size, Point};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClientMessage {
    WindowCreate { id: WindowId, size: Size },
    WindowDestroy { id: WindowId },
    BufferCommit { id: WindowId, data: Vec<u8> },
    WindowResize { id: WindowId, size: Size },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ServerMessage {
    KeyEvent { window_id: WindowId, scancode: u32, pressed: bool },
    PointerMove { window_id: WindowId, position: Point },
    PointerButton { window_id: WindowId, button: u32, pressed: bool },
    WindowFocus { window_id: WindowId },
    WindowClose { window_id: WindowId },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_message_roundtrip_bincode() {
        let msg = ClientMessage::WindowCreate {
            id: WindowId(42),
            size: Size { width: 1280, height: 720 },
        };
        let encoded = bincode::serde::encode_to_vec(&msg, bincode::config::standard()).unwrap();
        let (decoded, _): (ClientMessage, _) =
            bincode::serde::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn server_message_roundtrip_bincode() {
        let msg = ServerMessage::KeyEvent {
            window_id: WindowId(1),
            scancode: 65,
            pressed: true,
        };
        let encoded = bincode::serde::encode_to_vec(&msg, bincode::config::standard()).unwrap();
        let (decoded, _): (ServerMessage, _) =
            bincode::serde::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn buffer_commit_with_data() {
        let data = vec![0u8, 128, 255, 64];
        let msg = ClientMessage::BufferCommit {
            id: WindowId(7),
            data: data.clone(),
        };
        let encoded = bincode::serde::encode_to_vec(&msg, bincode::config::standard()).unwrap();
        let (decoded, _): (ClientMessage, _) =
            bincode::serde::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
        assert_eq!(msg, decoded);
        if let ClientMessage::BufferCommit { data: d, .. } = decoded {
            assert_eq!(d, data);
        }
    }
}
```

- [ ] **Step 2: Spusť testy — ověř že failing**

```bash
cargo test -p airproto messages 2>&1 | head -30
```

Očekávaný výstup: kompilační chyba `use of undeclared crate or module 'bincode'` nebo `cannot find module`.

- [ ] **Step 3: Spusť testy po přidání messages.rs do projektu**

```bash
cargo test -p airproto messages
```

Očekávaný výstup: 3 testy passing — `client_message_roundtrip_bincode`, `server_message_roundtrip_bincode`, `buffer_commit_with_data`.

- [ ] **Step 4: Commit**

```bash
git add crates/airproto/src/messages.rs
git commit -m "feat(airproto): add ClientMessage and ServerMessage enums with bincode serialization"
```

---

## Task 4: AirProto — connection (framed Unix socket)

**Files:**
- Create: `crates/airproto/src/connection.rs`

Protokol přenáší zprávy přes Unix socket jako **délkově-prefixované framy**: `[u32 LE délka][N bytů bincode payload]`. To umožňuje číst přesně jednu zprávu bez buffering problémů.

- [ ] **Step 1: Napiš failing test pro framed connection**

```rust
// crates/airproto/src/connection.rs
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use serde::{Serialize, de::DeserializeOwned};

pub struct AirProtoConn {
    stream: UnixStream,
}

impl AirProtoConn {
    pub async fn connect(path: &Path) -> std::io::Result<Self> {
        let stream = UnixStream::connect(path).await?;
        Ok(Self { stream })
    }

    pub fn from_stream(stream: UnixStream) -> Self {
        Self { stream }
    }

    /// Odešle zprávu jako length-prefixed frame.
    pub async fn send<T: Serialize>(&mut self, msg: &T) -> std::io::Result<()> {
        let payload = bincode::serde::encode_to_vec(msg, bincode::config::standard())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let len = payload.len() as u32;
        self.stream.write_all(&len.to_le_bytes()).await?;
        self.stream.write_all(&payload).await?;
        Ok(())
    }

    /// Přečte jednu zprávu z length-prefixed frame.
    pub async fn recv<T: DeserializeOwned>(&mut self) -> std::io::Result<T> {
        let mut len_buf = [0u8; 4];
        self.stream.read_exact(&mut len_buf).await?;
        let len = u32::from_le_bytes(len_buf) as usize;
        let mut payload = vec![0u8; len];
        self.stream.read_exact(&mut payload).await?;
        let (msg, _) = bincode::serde::decode_from_slice(&payload, bincode::config::standard())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::{ClientMessage, ServerMessage};
    use crate::types::{WindowId, Size};
    use tempfile::TempDir;

    #[tokio::test]
    async fn send_and_recv_client_message() {
        let dir = TempDir::new().unwrap();
        let socket_path = dir.path().join("test.sock");

        let listener = UnixListener::bind(&socket_path).unwrap();

        let path = socket_path.clone();
        let client_task = tokio::spawn(async move {
            let mut conn = AirProtoConn::connect(&path).await.unwrap();
            let msg = ClientMessage::WindowCreate {
                id: WindowId(1),
                size: Size { width: 800, height: 600 },
            };
            conn.send(&msg).await.unwrap();
        });

        let (server_stream, _) = listener.accept().await.unwrap();
        let mut server_conn = AirProtoConn::from_stream(server_stream);
        let received: ClientMessage = server_conn.recv().await.unwrap();

        client_task.await.unwrap();

        assert_eq!(
            received,
            ClientMessage::WindowCreate {
                id: WindowId(1),
                size: Size { width: 800, height: 600 },
            }
        );
    }

    #[tokio::test]
    async fn send_and_recv_server_message() {
        let dir = TempDir::new().unwrap();
        let socket_path = dir.path().join("test2.sock");

        let listener = UnixListener::bind(&socket_path).unwrap();

        let path = socket_path.clone();
        let server_task = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut conn = AirProtoConn::from_stream(stream);
            let msg = ServerMessage::WindowClose { window_id: WindowId(99) };
            conn.send(&msg).await.unwrap();
        });

        let mut client_conn = AirProtoConn::connect(&socket_path).await.unwrap();
        let received: ServerMessage = client_conn.recv().await.unwrap();

        server_task.await.unwrap();

        assert_eq!(received, ServerMessage::WindowClose { window_id: WindowId(99) });
    }

    #[tokio::test]
    async fn multiple_messages_in_sequence() {
        let dir = TempDir::new().unwrap();
        let socket_path = dir.path().join("test3.sock");

        let listener = UnixListener::bind(&socket_path).unwrap();

        let path = socket_path.clone();
        let client_task = tokio::spawn(async move {
            let mut conn = AirProtoConn::connect(&path).await.unwrap();
            conn.send(&ClientMessage::WindowCreate {
                id: WindowId(1),
                size: Size { width: 100, height: 100 },
            }).await.unwrap();
            conn.send(&ClientMessage::WindowDestroy { id: WindowId(1) }).await.unwrap();
        });

        let (stream, _) = listener.accept().await.unwrap();
        let mut server_conn = AirProtoConn::from_stream(stream);
        let m1: ClientMessage = server_conn.recv().await.unwrap();
        let m2: ClientMessage = server_conn.recv().await.unwrap();

        client_task.await.unwrap();

        assert!(matches!(m1, ClientMessage::WindowCreate { .. }));
        assert!(matches!(m2, ClientMessage::WindowDestroy { .. }));
    }
}
```

- [ ] **Step 2: Spusť testy — ověř že failing**

```bash
cargo test -p airproto connection 2>&1 | head -30
```

Očekávaný výstup: kompilační chyba protože `connection.rs` soubor zatím neexistuje v projektu.

- [ ] **Step 3: Spusť testy po přidání connection.rs**

```bash
cargo test -p airproto connection
```

Očekávaný výstup: 3 testy passing — `send_and_recv_client_message`, `send_and_recv_server_message`, `multiple_messages_in_sequence`.

- [ ] **Step 4: Spusť všechny airproto testy**

```bash
cargo test -p airproto
```

Očekávaný výstup: 8 testů passing, 0 failing.

- [ ] **Step 5: Commit**

```bash
git add crates/airproto/src/connection.rs
git commit -m "feat(airproto): add AirProtoConn with length-prefixed framing over Unix socket"
```

---

## Task 5: AirBus — zprávy

**Files:**
- Create: `crates/airbus/src/lib.rs`
- Create: `crates/airbus/src/messages.rs`

- [ ] **Step 1: Napiš failing test pro AirBus zprávy**

```rust
// crates/airbus/src/messages.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotificationPayload {
    pub title: String,
    pub body: String,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MenuItem {
    pub id: String,
    pub label: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BusMessage {
    Notify(NotificationPayload),
    SetDockBadge { count: u32 },
    RegisterMenu { items: Vec<MenuItem> },
    MenuItemClicked { id: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notify_roundtrip_msgpack() {
        let msg = BusMessage::Notify(NotificationPayload {
            title: "Test".to_string(),
            body: "Hello".to_string(),
            icon: None,
        });
        let encoded = rmp_serde::to_vec(&msg).unwrap();
        let decoded: BusMessage = rmp_serde::from_slice(&encoded).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn dock_badge_roundtrip_msgpack() {
        let msg = BusMessage::SetDockBadge { count: 5 };
        let encoded = rmp_serde::to_vec(&msg).unwrap();
        let decoded: BusMessage = rmp_serde::from_slice(&encoded).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn register_menu_roundtrip_msgpack() {
        let msg = BusMessage::RegisterMenu {
            items: vec![
                MenuItem { id: "open".to_string(), label: "Open".to_string(), enabled: true },
                MenuItem { id: "close".to_string(), label: "Close".to_string(), enabled: false },
            ],
        };
        let encoded = rmp_serde::to_vec(&msg).unwrap();
        let decoded: BusMessage = rmp_serde::from_slice(&encoded).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn notify_with_icon() {
        let msg = BusMessage::Notify(NotificationPayload {
            title: "Update".to_string(),
            body: "New version available".to_string(),
            icon: Some("/assets/icons/update.png".to_string()),
        });
        let encoded = rmp_serde::to_vec(&msg).unwrap();
        let decoded: BusMessage = rmp_serde::from_slice(&encoded).unwrap();
        if let BusMessage::Notify(p) = decoded {
            assert_eq!(p.icon, Some("/assets/icons/update.png".to_string()));
        } else {
            panic!("wrong variant");
        }
    }
}
```

- [ ] **Step 2: Vytvoř airbus lib.rs**

```rust
// crates/airbus/src/lib.rs
pub mod messages;
pub mod connection;

pub use messages::{BusMessage, NotificationPayload, MenuItem};
pub use connection::AirBusConn;
```

- [ ] **Step 3: Spusť testy — ověř že failing**

```bash
cargo test -p airbus messages 2>&1 | head -30
```

Očekávaný výstup: chyba `cannot find module 'connection'` nebo chybějící `rmp_serde` import.

- [ ] **Step 4: Spusť testy po přidání messages.rs**

```bash
cargo test -p airbus messages
```

Očekávaný výstup: 4 testy passing.

- [ ] **Step 5: Commit**

```bash
git add crates/airbus/src/
git commit -m "feat(airbus): add BusMessage enum with MessagePack serialization"
```

---

## Task 6: AirBus — connection

**Files:**
- Create: `crates/airbus/src/connection.rs`

AirBus používá stejný length-prefix framing jako AirProto, ale payload je MessagePack místo bincode.

- [ ] **Step 1: Napiš failing test pro AirBus connection**

```rust
// crates/airbus/src/connection.rs
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use crate::messages::BusMessage;

pub struct AirBusConn {
    stream: UnixStream,
}

impl AirBusConn {
    pub async fn connect(path: &Path) -> std::io::Result<Self> {
        let stream = UnixStream::connect(path).await?;
        Ok(Self { stream })
    }

    pub fn from_stream(stream: UnixStream) -> Self {
        Self { stream }
    }

    pub async fn send(&mut self, msg: &BusMessage) -> std::io::Result<()> {
        let payload = rmp_serde::to_vec(msg)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let len = payload.len() as u32;
        self.stream.write_all(&len.to_le_bytes()).await?;
        self.stream.write_all(&payload).await?;
        Ok(())
    }

    pub async fn recv(&mut self) -> std::io::Result<BusMessage> {
        let mut len_buf = [0u8; 4];
        self.stream.read_exact(&mut len_buf).await?;
        let len = u32::from_le_bytes(len_buf) as usize;
        let mut payload = vec![0u8; len];
        self.stream.read_exact(&mut payload).await?;
        rmp_serde::from_slice(&payload)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::NotificationPayload;
    use tempfile::TempDir;

    #[tokio::test]
    async fn send_notify_over_socket() {
        let dir = TempDir::new().unwrap();
        let socket_path = dir.path().join("airbus.sock");

        let listener = UnixListener::bind(&socket_path).unwrap();

        let path = socket_path.clone();
        let client_task = tokio::spawn(async move {
            let mut conn = AirBusConn::connect(&path).await.unwrap();
            conn.send(&BusMessage::Notify(NotificationPayload {
                title: "Hello".to_string(),
                body: "World".to_string(),
                icon: None,
            })).await.unwrap();
        });

        let (stream, _) = listener.accept().await.unwrap();
        let mut server_conn = AirBusConn::from_stream(stream);
        let received = server_conn.recv().await.unwrap();

        client_task.await.unwrap();

        assert!(matches!(received, BusMessage::Notify(_)));
    }

    #[tokio::test]
    async fn send_dock_badge_over_socket() {
        let dir = TempDir::new().unwrap();
        let socket_path = dir.path().join("airbus2.sock");

        let listener = UnixListener::bind(&socket_path).unwrap();

        let path = socket_path.clone();
        let client_task = tokio::spawn(async move {
            let mut conn = AirBusConn::connect(&path).await.unwrap();
            conn.send(&BusMessage::SetDockBadge { count: 3 }).await.unwrap();
        });

        let (stream, _) = listener.accept().await.unwrap();
        let mut server_conn = AirBusConn::from_stream(stream);
        let received = server_conn.recv().await.unwrap();

        client_task.await.unwrap();

        assert_eq!(received, BusMessage::SetDockBadge { count: 3 });
    }
}
```

- [ ] **Step 2: Spusť testy — ověř že failing**

```bash
cargo test -p airbus connection 2>&1 | head -30
```

Očekávaný výstup: kompilační chyba protože `connection.rs` ještě neexistuje.

- [ ] **Step 3: Spusť testy po přidání connection.rs**

```bash
cargo test -p airbus connection
```

Očekávaný výstup: 2 testy passing.

- [ ] **Step 4: Spusť všechny testy v workspace**

```bash
cargo test
```

Očekávaný výstup: 14 testů passing, 0 failing.

- [ ] **Step 5: Commit**

```bash
git add crates/airbus/src/connection.rs
git commit -m "feat(airbus): add AirBusConn with length-prefixed MessagePack framing"
```

---

## Task 7: CI/CD setup

**Files:**
- Create: `.github/workflows/ci.yml`

- [ ] **Step 1: Vytvoř GitHub Actions workflow**

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: macos-latest
    strategy:
      matrix:
        target: [aarch64-apple-darwin, x86_64-apple-darwin]
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: aarch64-apple-darwin,x86_64-apple-darwin

      - name: Cache
        uses: Swatinem/rust-cache@v2

      - name: Check
        run: cargo check --target ${{ matrix.target }}

      - name: Test (native only)
        if: matrix.target == 'x86_64-apple-darwin'
        run: cargo test

      - name: Clippy
        run: cargo clippy --target ${{ matrix.target }} -- -D warnings

      - name: Format check
        run: cargo fmt --check
```

- [ ] **Step 2: Přidej rustfmt.toml**

```toml
# rustfmt.toml
edition = "2021"
max_width = 100
```

- [ ] **Step 3: Spusť fmt a clippy lokálně**

```bash
cargo fmt
cargo clippy -- -D warnings
```

Očekávaný výstup: žádná clippy varování. Pokud jsou varování, oprav je před commitem.

- [ ] **Step 4: Commit**

```bash
git add .github/ rustfmt.toml
git commit -m "ci: add GitHub Actions workflow for multi-arch build and test"
```

---

## Self-Review Checklist

- [x] **Spec coverage:** Workspace ✓, AirProto typy ✓, AirProto zprávy ✓, AirProto connection ✓, AirBus zprávy ✓, AirBus connection ✓, multiarch ✓
- [x] **Placeholders:** Žádné TBD/TODO
- [x] **Konzistence typů:** `WindowId`, `Size`, `Point` definovány v Task 2 a použity konzistentně v Task 3 a 4. `BusMessage`, `NotificationPayload`, `MenuItem` definovány v Task 5 a použity v Task 6.
- [x] **Bincode API:** Používá `bincode 2` API (`bincode::serde::encode_to_vec`, `decode_from_slice`) konzistentně — ne staré `bincode::serialize`.
