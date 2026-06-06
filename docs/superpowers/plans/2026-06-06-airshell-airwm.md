# AirShell + AirWM — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Vytvořit `airshell` (stav docku, notifikací, menu baru + napojení na AirBus) a `airwm` (správa oken, snap logika, workspaces) jako pure-state Rust crates bez GPU renderování.

**Architecture:** `airshell` přijímá `BusMessage` zprávy od aplikací a udržuje stav docku, fronty notifikací a menu baru. `airwm` spravuje rozmístění oken (floating + snap), z-order a virtuální plochy — vše jako datový model nezávislý na displayi. Renderování (wgpu draw cally) přijde v pozdějším plánu.

**Tech Stack:** Rust 1.78+, `airbus` (pro BusMessage/MenuItem), `airproto` (pro WindowId/Size/Point), žádné další závislosti.

---

## Soubory

```
crates/airshell/
├── Cargo.toml                  # depends on airbus
└── src/
    ├── lib.rs                  # re-exports
    ├── dock.rs                 # DockItem, DockState
    ├── notifications.rs        # NotificationEntry, NotificationQueue
    ├── topbar.rs               # TopBarState, ShellState
    └── bus_handler.rs          # handle_bus_message(ShellState, BusMessage)

crates/airwm/
├── Cargo.toml                  # depends on airproto
└── src/
    ├── lib.rs                  # re-exports
    ├── snap.rs                 # SnapZone, compute_snap, snap_rect
    ├── window.rs               # ManagedWindow, WindowState
    ├── manager.rs              # WindowManager
    └── workspace.rs            # Workspace, WorkspaceManager
```

---

## Task 1: airshell crate setup

**Files:**
- Modify: `Cargo.toml` (workspace root)
- Create: `crates/airshell/Cargo.toml`
- Create: `crates/airshell/src/lib.rs`
- Create stubs: `crates/airshell/src/dock.rs`, `crates/airshell/src/notifications.rs`, `crates/airshell/src/topbar.rs`, `crates/airshell/src/bus_handler.rs`

- [ ] **Step 1: Přidej `airshell` do workspace `Cargo.toml`**

```toml
[workspace]
resolver = "2"
members = [
    "crates/airproto",
    "crates/airbus",
    "crates/aircomp",
    "crates/airkit-theme",
    "crates/airkit-ui",
    "crates/airshell",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["AirOS Contributors"]

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
bincode = "2"
rmp-serde = "1"
tokio = { version = "1", features = ["net", "io-util", "macros", "rt-multi-thread", "test-util", "time"] }
tempfile = "3"
wgpu = "22"
```

- [ ] **Step 2: Vytvoř `crates/airshell/Cargo.toml`**

```toml
[package]
name = "airshell"
version.workspace = true
edition.workspace = true

[dependencies]
airbus = { path = "../airbus" }
```

- [ ] **Step 3: Vytvoř `crates/airshell/src/lib.rs`**

```rust
pub mod bus_handler;
pub mod dock;
pub mod notifications;
pub mod topbar;
```

- [ ] **Step 4: Vytvoř prázdné stub soubory**

- `crates/airshell/src/dock.rs` — prázdný soubor
- `crates/airshell/src/notifications.rs` — prázdný soubor
- `crates/airshell/src/topbar.rs` — prázdný soubor
- `crates/airshell/src/bus_handler.rs` — prázdný soubor

- [ ] **Step 5: Ověř build**

```bash
cd /Users/vojtechprouza/projekty/AirOS && cargo check -p airshell 2>&1 | tail -3
```

Očekávaný výstup: `Finished dev profile` (prázdné moduly se kompilují bez chyb).

- [ ] **Step 6: Commit**

```bash
cd /Users/vojtechprouza/projekty/AirOS
git add Cargo.toml crates/airshell/
git commit -m "chore(airshell): add airshell crate to workspace"
```

---

## Task 2: DockState

**Files:**
- Modify: `crates/airshell/src/dock.rs`

- [ ] **Step 1: Napiš implementaci s testy**

```rust
// crates/airshell/src/dock.rs

#[derive(Debug, Clone, PartialEq)]
pub struct DockItem {
    pub id: String,
    pub label: String,
    pub is_running: bool,
    pub is_pinned: bool,
    pub badge: Option<u32>,
}

impl DockItem {
    fn new(id: impl Into<String>, label: impl Into<String>, pinned: bool, running: bool) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            is_running: running,
            is_pinned: pinned,
            badge: None,
        }
    }
}

/// Stav docku: pinned aplikace + spuštěné aplikace.
#[derive(Debug, Default)]
pub struct DockState {
    items: Vec<DockItem>,
}

impl DockState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Přidá/označí položku jako pinned.
    pub fn pin(&mut self, id: impl Into<String>, label: impl Into<String>) {
        let id = id.into();
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.is_pinned = true;
        } else {
            self.items.push(DockItem::new(id, label, true, false));
        }
    }

    /// Odstraní pin. Pokud aplikace neběží, odstraní ji z docku úplně.
    pub fn unpin(&mut self, id: &str) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.is_pinned = false;
            if !item.is_running {
                self.items.retain(|i| i.id != id);
            }
        }
    }

    /// Nastaví running stav aplikace. Pokud running=true a položka neexistuje, přidá ji.
    /// Pokud running=false a položka není pinned, odstraní ji.
    pub fn set_running(&mut self, id: &str, label: &str, running: bool) {
        if running {
            if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
                item.is_running = true;
            } else {
                self.items.push(DockItem::new(id, label, false, true));
            }
        } else if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.is_running = false;
            if !item.is_pinned {
                self.items.retain(|i| i.id != id);
            }
        }
    }

    /// Nastaví badge count. count=0 odstraní badge. No-op pokud item neexistuje.
    pub fn set_badge(&mut self, id: &str, count: u32) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.badge = if count > 0 { Some(count) } else { None };
        }
    }

    pub fn get(&self, id: &str) -> Option<&DockItem> {
        self.items.iter().find(|i| i.id == id)
    }

    pub fn items(&self) -> &[DockItem] {
        &self.items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pin_adds_item_to_dock() {
        let mut d = DockState::new();
        d.pin("finder", "Finder");
        let item = d.get("finder").unwrap();
        assert!(item.is_pinned);
        assert!(!item.is_running);
        assert_eq!(item.label, "Finder");
    }

    #[test]
    fn pin_existing_item_marks_it_pinned() {
        let mut d = DockState::new();
        d.set_running("finder", "Finder", true);
        d.pin("finder", "Finder");
        let item = d.get("finder").unwrap();
        assert!(item.is_pinned);
        assert!(item.is_running);
    }

    #[test]
    fn set_running_creates_unpinned_item() {
        let mut d = DockState::new();
        d.set_running("term", "Terminal", true);
        let item = d.get("term").unwrap();
        assert!(item.is_running);
        assert!(!item.is_pinned);
    }

    #[test]
    fn set_running_false_removes_unpinned_item() {
        let mut d = DockState::new();
        d.set_running("term", "Terminal", true);
        d.set_running("term", "Terminal", false);
        assert!(d.get("term").is_none());
    }

    #[test]
    fn set_running_false_keeps_pinned_item() {
        let mut d = DockState::new();
        d.pin("finder", "Finder");
        d.set_running("finder", "Finder", true);
        d.set_running("finder", "Finder", false);
        let item = d.get("finder").unwrap();
        assert!(item.is_pinned);
        assert!(!item.is_running);
    }

    #[test]
    fn set_badge_updates_badge_count() {
        let mut d = DockState::new();
        d.set_running("mail", "Mail", true);
        d.set_badge("mail", 5);
        assert_eq!(d.get("mail").unwrap().badge, Some(5));
    }

    #[test]
    fn set_badge_zero_clears_badge() {
        let mut d = DockState::new();
        d.set_running("mail", "Mail", true);
        d.set_badge("mail", 3);
        d.set_badge("mail", 0);
        assert_eq!(d.get("mail").unwrap().badge, None);
    }
}
```

- [ ] **Step 2: Spusť testy**

```bash
cd /Users/vojtechprouza/projekty/AirOS && cargo test -p airshell dock 2>&1 | tail -5
```

Očekávaný výstup: `test result: ok. 7 passed`

- [ ] **Step 3: Commit**

```bash
cd /Users/vojtechprouza/projekty/AirOS
git add crates/airshell/src/dock.rs
git commit -m "feat(airshell): add DockState with pinned/running items and badge support"
```

---

## Task 3: NotificationQueue

**Files:**
- Modify: `crates/airshell/src/notifications.rs`

- [ ] **Step 1: Napiš implementaci s testy**

```rust
// crates/airshell/src/notifications.rs

use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub struct NotificationEntry {
    pub id: u64,
    pub title: String,
    pub body: String,
    pub icon: Option<String>,
}

/// Fronta notifikací s omezenou kapacitou (FIFO, oldest first).
pub struct NotificationQueue {
    entries: VecDeque<NotificationEntry>,
    max_size: usize,
    next_id: u64,
}

impl NotificationQueue {
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            max_size,
            next_id: 1,
        }
    }

    /// Přidá notifikaci a vrátí její ID. Překročení max_size odstraní nejstarší.
    pub fn push(&mut self, title: String, body: String, icon: Option<String>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.entries
            .push_back(NotificationEntry { id, title, body, icon });
        if self.entries.len() > self.max_size {
            self.entries.pop_front();
        }
        id
    }

    /// Odstraní notifikaci dle ID. Vrátí true pokud existovala.
    pub fn dismiss(&mut self, id: u64) -> bool {
        let before = self.entries.len();
        self.entries.retain(|e| e.id != id);
        self.entries.len() < before
    }

    pub fn entries(&self) -> &VecDeque<NotificationEntry> {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_queue_is_empty() {
        let q = NotificationQueue::new(10);
        assert!(q.is_empty());
        assert_eq!(q.len(), 0);
    }

    #[test]
    fn push_returns_incrementing_ids() {
        let mut q = NotificationQueue::new(10);
        let id1 = q.push("A".into(), "body".into(), None);
        let id2 = q.push("B".into(), "body".into(), None);
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(q.len(), 2);
    }

    #[test]
    fn push_beyond_max_drops_oldest() {
        let mut q = NotificationQueue::new(2);
        let id1 = q.push("A".into(), "b".into(), None);
        q.push("B".into(), "b".into(), None);
        q.push("C".into(), "b".into(), None); // drops id1
        assert_eq!(q.len(), 2);
        assert!(q.entries().iter().all(|e| e.id != id1));
    }

    #[test]
    fn dismiss_removes_entry() {
        let mut q = NotificationQueue::new(10);
        let id = q.push("Hello".into(), "World".into(), None);
        let removed = q.dismiss(id);
        assert!(removed);
        assert!(q.is_empty());
    }

    #[test]
    fn dismiss_nonexistent_returns_false() {
        let mut q = NotificationQueue::new(10);
        assert!(!q.dismiss(999));
    }
}
```

- [ ] **Step 2: Spusť testy**

```bash
cd /Users/vojtechprouza/projekty/AirOS && cargo test -p airshell notifications 2>&1 | tail -5
```

Očekávaný výstup: `test result: ok. 5 passed`

- [ ] **Step 3: Commit**

```bash
cd /Users/vojtechprouza/projekty/AirOS
git add crates/airshell/src/notifications.rs
git commit -m "feat(airshell): add NotificationQueue with max-size eviction"
```

---

## Task 4: TopBarState + ShellState

**Files:**
- Modify: `crates/airshell/src/topbar.rs`

- [ ] **Step 1: Napiš implementaci s testy**

```rust
// crates/airshell/src/topbar.rs

use airbus::messages::MenuItem;
use crate::dock::DockState;
use crate::notifications::NotificationQueue;
use std::collections::HashMap;

/// Stav top baru: registrovaná menu aplikací, aktivní aplikace.
#[derive(Debug, Default)]
pub struct TopBarState {
    registered_menus: HashMap<String, Vec<MenuItem>>,
    active_app_id: Option<String>,
}

impl TopBarState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Zaregistruje menu pro danou aplikaci.
    pub fn register_menu(&mut self, app_id: String, items: Vec<MenuItem>) {
        self.registered_menus.insert(app_id, items);
    }

    /// Odstraní menu aplikace (při ukončení).
    pub fn unregister_menu(&mut self, app_id: &str) {
        self.registered_menus.remove(app_id);
    }

    /// Nastaví aktivní aplikaci (focus).
    pub fn set_active_app(&mut self, app_id: Option<String>) {
        self.active_app_id = app_id;
    }

    /// Vrátí menu položky pro aktivní aplikaci.
    pub fn active_menu(&self) -> Option<&[MenuItem]> {
        self.active_app_id
            .as_ref()
            .and_then(|id| self.registered_menus.get(id))
            .map(Vec::as_slice)
    }
}

/// Kompletní stav AirShell: dock + notifikace + top bar.
pub struct ShellState {
    pub dock: DockState,
    pub notifications: NotificationQueue,
    pub topbar: TopBarState,
}

impl ShellState {
    pub fn new() -> Self {
        Self {
            dock: DockState::new(),
            notifications: NotificationQueue::new(20),
            topbar: TopBarState::new(),
        }
    }
}

impl Default for ShellState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_menu() -> Vec<MenuItem> {
        vec![
            MenuItem { id: "new".into(), label: "New".into(), enabled: true },
            MenuItem { id: "quit".into(), label: "Quit".into(), enabled: true },
        ]
    }

    #[test]
    fn register_menu_stores_items() {
        let mut t = TopBarState::new();
        t.register_menu("finder".into(), make_menu());
        assert!(t.registered_menus.contains_key("finder"));
    }

    #[test]
    fn active_menu_none_when_no_active_app() {
        let t = TopBarState::new();
        assert!(t.active_menu().is_none());
    }

    #[test]
    fn active_menu_returns_items_for_active_app() {
        let mut t = TopBarState::new();
        t.register_menu("finder".into(), make_menu());
        t.set_active_app(Some("finder".into()));
        let menu = t.active_menu().unwrap();
        assert_eq!(menu.len(), 2);
        assert_eq!(menu[0].id, "new");
    }

    #[test]
    fn unregister_menu_removes_it() {
        let mut t = TopBarState::new();
        t.register_menu("finder".into(), make_menu());
        t.unregister_menu("finder");
        assert!(!t.registered_menus.contains_key("finder"));
    }

    #[test]
    fn shell_state_new_dock_and_queue_are_empty() {
        let s = ShellState::new();
        assert_eq!(s.dock.items().len(), 0);
        assert!(s.notifications.is_empty());
    }
}
```

- [ ] **Step 2: Spusť testy**

```bash
cd /Users/vojtechprouza/projekty/AirOS && cargo test -p airshell topbar 2>&1 | tail -5
```

Očekávaný výstup: `test result: ok. 5 passed`

- [ ] **Step 3: Commit**

```bash
cd /Users/vojtechprouza/projekty/AirOS
git add crates/airshell/src/topbar.rs
git commit -m "feat(airshell): add TopBarState and ShellState"
```

---

## Task 5: bus_handler + lib.rs wire-up (airshell)

**Files:**
- Modify: `crates/airshell/src/bus_handler.rs`
- Modify: `crates/airshell/src/lib.rs`

- [ ] **Step 1: Napiš `bus_handler.rs`**

```rust
// crates/airshell/src/bus_handler.rs

use airbus::messages::{BusMessage, NotificationPayload};
use crate::topbar::ShellState;

/// Zpracuje příchozí BusMessage od aplikace a aktualizuje ShellState.
/// `app_id` identifikuje odesílatele (název/ID aplikace).
pub fn handle_bus_message(state: &mut ShellState, app_id: &str, msg: BusMessage) {
    match msg {
        BusMessage::Notify(NotificationPayload { title, body, icon }) => {
            state.notifications.push(title, body, icon);
        }
        BusMessage::SetDockBadge { count } => {
            state.dock.set_badge(app_id, count);
        }
        BusMessage::RegisterMenu { items } => {
            state.topbar.register_menu(app_id.to_string(), items);
        }
        BusMessage::MenuItemClicked { .. } => {
            // Odchozí zpráva (AirShell → app) — příchozím směrem ignorujeme.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use airbus::messages::{MenuItem, NotificationPayload};

    #[test]
    fn handle_notify_pushes_notification() {
        let mut state = ShellState::new();
        handle_bus_message(
            &mut state,
            "myapp",
            BusMessage::Notify(NotificationPayload {
                title: "Hello".into(),
                body: "World".into(),
                icon: None,
            }),
        );
        assert_eq!(state.notifications.len(), 1);
        let entry = state.notifications.entries().front().unwrap();
        assert_eq!(entry.title, "Hello");
        assert_eq!(entry.body, "World");
    }

    #[test]
    fn handle_set_dock_badge_updates_dock() {
        let mut state = ShellState::new();
        // Aplikace musí být v docku než může mít badge
        state.dock.set_running("myapp", "My App", true);
        handle_bus_message(&mut state, "myapp", BusMessage::SetDockBadge { count: 3 });
        assert_eq!(state.dock.get("myapp").unwrap().badge, Some(3));
    }

    #[test]
    fn handle_register_menu_updates_topbar() {
        let mut state = ShellState::new();
        handle_bus_message(
            &mut state,
            "finder",
            BusMessage::RegisterMenu {
                items: vec![MenuItem {
                    id: "open".into(),
                    label: "Open".into(),
                    enabled: true,
                }],
            },
        );
        state.topbar.set_active_app(Some("finder".into()));
        let menu = state.topbar.active_menu().unwrap();
        assert_eq!(menu.len(), 1);
        assert_eq!(menu[0].id, "open");
    }
}
```

- [ ] **Step 2: Aktualizuj `crates/airshell/src/lib.rs`**

```rust
pub mod bus_handler;
pub mod dock;
pub mod notifications;
pub mod topbar;

pub use bus_handler::handle_bus_message;
pub use dock::{DockItem, DockState};
pub use notifications::{NotificationEntry, NotificationQueue};
pub use topbar::{ShellState, TopBarState};
```

- [ ] **Step 3: Spusť všechny airshell testy**

```bash
cd /Users/vojtechprouza/projekty/AirOS && cargo test -p airshell 2>&1 | tail -5
```

Očekávaný výstup: `test result: ok. 20 passed` (7 dock + 5 notifications + 5 topbar + 3 bus_handler)

- [ ] **Step 4: Clippy**

```bash
cd /Users/vojtechprouza/projekty/AirOS && cargo clippy -p airshell -- -D warnings 2>&1 | tail -3
```

Očekávaný výstup: `Finished dev profile` bez warningů.

- [ ] **Step 5: Commit**

```bash
cd /Users/vojtechprouza/projekty/AirOS
git add crates/airshell/src/bus_handler.rs crates/airshell/src/lib.rs
git commit -m "feat(airshell): add bus_handler and wire up all re-exports in lib.rs"
```

---

## Task 6: airwm crate setup

**Files:**
- Modify: `Cargo.toml` (workspace root)
- Create: `crates/airwm/Cargo.toml`
- Create: `crates/airwm/src/lib.rs`
- Create stubs: `crates/airwm/src/snap.rs`, `crates/airwm/src/window.rs`, `crates/airwm/src/manager.rs`, `crates/airwm/src/workspace.rs`

- [ ] **Step 1: Přidej `airwm` do workspace `Cargo.toml`**

```toml
[workspace]
resolver = "2"
members = [
    "crates/airproto",
    "crates/airbus",
    "crates/aircomp",
    "crates/airkit-theme",
    "crates/airkit-ui",
    "crates/airshell",
    "crates/airwm",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["AirOS Contributors"]

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
bincode = "2"
rmp-serde = "1"
tokio = { version = "1", features = ["net", "io-util", "macros", "rt-multi-thread", "test-util", "time"] }
tempfile = "3"
wgpu = "22"
```

- [ ] **Step 2: Vytvoř `crates/airwm/Cargo.toml`**

```toml
[package]
name = "airwm"
version.workspace = true
edition.workspace = true

[dependencies]
airproto = { path = "../airproto" }
```

- [ ] **Step 3: Vytvoř `crates/airwm/src/lib.rs`**

```rust
pub mod manager;
pub mod snap;
pub mod window;
pub mod workspace;
```

- [ ] **Step 4: Vytvoř prázdné stub soubory**

- `crates/airwm/src/snap.rs` — prázdný soubor
- `crates/airwm/src/window.rs` — prázdný soubor
- `crates/airwm/src/manager.rs` — prázdný soubor
- `crates/airwm/src/workspace.rs` — prázdný soubor

- [ ] **Step 5: Ověř build**

```bash
cd /Users/vojtechprouza/projekty/AirOS && cargo check -p airwm 2>&1 | tail -3
```

Očekávaný výstup: `Finished dev profile`

- [ ] **Step 6: Commit**

```bash
cd /Users/vojtechprouza/projekty/AirOS
git add Cargo.toml crates/airwm/
git commit -m "chore(airwm): add airwm crate to workspace"
```

---

## Task 7: Snap logika

**Files:**
- Modify: `crates/airwm/src/snap.rs`

Snap zóny: Left/Right (polovina obrazovky), TopLeft/TopRight/BottomLeft/BottomRight (čtvrtina). Rohové zóny mají prioritu před hranami.

- [ ] **Step 1: Napiš implementaci s testy**

```rust
// crates/airwm/src/snap.rs

/// Zóna pro snap okna.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SnapZone {
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// Vrátí SnapZone pokud je kurzor v oblasti snap triggeru (threshold od hrany/rohu).
/// Rohy mají prioritu před hranami.
pub fn compute_snap(
    cursor_x: f32,
    cursor_y: f32,
    screen_w: f32,
    screen_h: f32,
    threshold: f32,
) -> Option<SnapZone> {
    let near_left = cursor_x < threshold;
    let near_right = cursor_x > screen_w - threshold;
    let near_top = cursor_y < threshold;
    let near_bottom = cursor_y > screen_h - threshold;

    match (near_left, near_right, near_top, near_bottom) {
        (true, _, true, _) => Some(SnapZone::TopLeft),
        (true, _, _, true) => Some(SnapZone::BottomLeft),
        (_, true, true, _) => Some(SnapZone::TopRight),
        (_, true, _, true) => Some(SnapZone::BottomRight),
        (true, _, _, _) => Some(SnapZone::Left),
        (_, true, _, _) => Some(SnapZone::Right),
        _ => None,
    }
}

/// Vrátí cílový rect (x, y, width, height) pro danou snap zónu a velikost obrazovky.
pub fn snap_rect(zone: SnapZone, screen_w: f32, screen_h: f32) -> (f32, f32, f32, f32) {
    let hw = screen_w / 2.0;
    let hh = screen_h / 2.0;
    match zone {
        SnapZone::Left => (0.0, 0.0, hw, screen_h),
        SnapZone::Right => (hw, 0.0, hw, screen_h),
        SnapZone::TopLeft => (0.0, 0.0, hw, hh),
        SnapZone::TopRight => (hw, 0.0, hw, hh),
        SnapZone::BottomLeft => (0.0, hh, hw, hh),
        SnapZone::BottomRight => (hw, hh, hw, hh),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const W: f32 = 1920.0;
    const H: f32 = 1080.0;
    const T: f32 = 20.0; // threshold

    #[test]
    fn compute_snap_near_left_edge() {
        let zone = compute_snap(5.0, 540.0, W, H, T);
        assert_eq!(zone, Some(SnapZone::Left));
    }

    #[test]
    fn compute_snap_near_right_edge() {
        let zone = compute_snap(1915.0, 540.0, W, H, T);
        assert_eq!(zone, Some(SnapZone::Right));
    }

    #[test]
    fn compute_snap_top_left_corner_takes_priority() {
        // Blízko levé hrany I top hrany → rohová zóna vyhraje
        let zone = compute_snap(5.0, 5.0, W, H, T);
        assert_eq!(zone, Some(SnapZone::TopLeft));
    }

    #[test]
    fn compute_snap_bottom_right_corner() {
        let zone = compute_snap(1915.0, 1075.0, W, H, T);
        assert_eq!(zone, Some(SnapZone::BottomRight));
    }

    #[test]
    fn compute_snap_middle_returns_none() {
        let zone = compute_snap(960.0, 540.0, W, H, T);
        assert!(zone.is_none());
    }

    #[test]
    fn snap_rect_left_half_width() {
        let (x, y, w, h) = snap_rect(SnapZone::Left, 1920.0, 1080.0);
        assert!((x - 0.0).abs() < f32::EPSILON);
        assert!((y - 0.0).abs() < f32::EPSILON);
        assert!((w - 960.0).abs() < f32::EPSILON);
        assert!((h - 1080.0).abs() < f32::EPSILON);
    }

    #[test]
    fn snap_rect_top_right_quarter() {
        let (x, y, w, h) = snap_rect(SnapZone::TopRight, 1920.0, 1080.0);
        assert!((x - 960.0).abs() < f32::EPSILON);
        assert!((y - 0.0).abs() < f32::EPSILON);
        assert!((w - 960.0).abs() < f32::EPSILON);
        assert!((h - 540.0).abs() < f32::EPSILON);
    }
}
```

- [ ] **Step 2: Spusť testy**

```bash
cd /Users/vojtechprouza/projekty/AirOS && cargo test -p airwm snap 2>&1 | tail -5
```

Očekávaný výstup: `test result: ok. 7 passed`

- [ ] **Step 3: Commit**

```bash
cd /Users/vojtechprouza/projekty/AirOS
git add crates/airwm/src/snap.rs
git commit -m "feat(airwm): add SnapZone, compute_snap and snap_rect"
```

---

## Task 8: ManagedWindow

**Files:**
- Modify: `crates/airwm/src/window.rs`

`ManagedWindow` drží stav jednoho okna: pozici, velikost, titulek, stav (Normal/Minimized/Maximized/Snapped). Volání `snap_to` deleguje výpočet rectu na `crate::snap::snap_rect`.

- [ ] **Step 1: Napiš implementaci s testy**

```rust
// crates/airwm/src/window.rs

use airproto::types::{Point, Size, WindowId};
use crate::snap::{snap_rect, SnapZone};

/// Stav okna.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowState {
    Normal,
    Minimized,
    Maximized,
    Snapped(SnapZone),
}

/// Spravované okno v AirWM.
#[derive(Debug, Clone)]
pub struct ManagedWindow {
    pub id: WindowId,
    pub title: String,
    pub position: Point,
    pub size: Size,
    pub state: WindowState,
}

impl ManagedWindow {
    pub fn new(id: WindowId, title: impl Into<String>, position: Point, size: Size) -> Self {
        Self {
            id,
            title: title.into(),
            position,
            size,
            state: WindowState::Normal,
        }
    }

    /// Přesune okno. Ignoruje se pokud není Normal.
    pub fn move_to(&mut self, position: Point) {
        if self.state == WindowState::Normal {
            self.position = position;
        }
    }

    /// Změní velikost. Ignoruje se pokud není Normal.
    pub fn resize_to(&mut self, size: Size) {
        if self.state == WindowState::Normal {
            self.size = size;
        }
    }

    pub fn minimize(&mut self) {
        self.state = WindowState::Minimized;
    }

    pub fn restore(&mut self) {
        self.state = WindowState::Normal;
    }

    pub fn maximize(&mut self, screen: Size) {
        self.state = WindowState::Maximized;
        self.position = Point { x: 0.0, y: 0.0 };
        self.size = screen;
    }

    /// Přichytí okno na danou snap zónu.
    pub fn snap_to(&mut self, zone: SnapZone, screen: Size) {
        let (x, y, w, h) = snap_rect(zone, screen.width as f32, screen.height as f32);
        self.state = WindowState::Snapped(zone);
        self.position = Point { x, y };
        self.size = Size {
            width: w as u32,
            height: h as u32,
        };
    }

    /// Vrátí false pokud je okno minimalizované.
    pub fn is_visible(&self) -> bool {
        self.state != WindowState::Minimized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_window() -> ManagedWindow {
        ManagedWindow::new(
            WindowId(1),
            "Test",
            Point { x: 100.0, y: 100.0 },
            Size { width: 800, height: 600 },
        )
    }

    fn screen() -> Size {
        Size { width: 1920, height: 1080 }
    }

    #[test]
    fn new_window_is_normal_and_visible() {
        let w = make_window();
        assert_eq!(w.state, WindowState::Normal);
        assert!(w.is_visible());
    }

    #[test]
    fn minimize_hides_window() {
        let mut w = make_window();
        w.minimize();
        assert_eq!(w.state, WindowState::Minimized);
        assert!(!w.is_visible());
    }

    #[test]
    fn restore_from_minimize() {
        let mut w = make_window();
        w.minimize();
        w.restore();
        assert_eq!(w.state, WindowState::Normal);
        assert!(w.is_visible());
    }

    #[test]
    fn maximize_fills_screen() {
        let mut w = make_window();
        w.maximize(screen());
        assert_eq!(w.state, WindowState::Maximized);
        assert_eq!(w.size.width, 1920);
        assert_eq!(w.size.height, 1080);
        assert!((w.position.x - 0.0).abs() < f32::EPSILON);
        assert!((w.position.y - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn snap_to_left_sets_half_width() {
        let mut w = make_window();
        w.snap_to(SnapZone::Left, screen());
        assert_eq!(w.state, WindowState::Snapped(SnapZone::Left));
        assert_eq!(w.size.width, 960);
        assert_eq!(w.size.height, 1080);
        assert!((w.position.x - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn move_to_only_works_in_normal_state() {
        let mut w = make_window();
        w.maximize(screen());
        w.move_to(Point { x: 50.0, y: 50.0 });
        // Pozice se nesmí změnit — okno je maximized
        assert!((w.position.x - 0.0).abs() < f32::EPSILON);
    }
}
```

- [ ] **Step 2: Spusť testy**

```bash
cd /Users/vojtechprouza/projekty/AirOS && cargo test -p airwm window 2>&1 | tail -5
```

Očekávaný výstup: `test result: ok. 6 passed`

- [ ] **Step 3: Commit**

```bash
cd /Users/vojtechprouza/projekty/AirOS
git add crates/airwm/src/window.rs
git commit -m "feat(airwm): add ManagedWindow with Normal/Minimized/Maximized/Snapped states"
```

---

## Task 9: WindowManager

**Files:**
- Modify: `crates/airwm/src/manager.rs`

`WindowManager` je centrální stav všech oken: HashMap + z-order Vec (jako `WindowScene` v aircomp, ale s plnou WM logikou). Drží `screen: Size` pro maximize a snap operace.

- [ ] **Step 1: Napiš implementaci s testy**

```rust
// crates/airwm/src/manager.rs

use std::collections::HashMap;
use airproto::types::{Point, Size, WindowId};
use crate::snap::SnapZone;
use crate::window::ManagedWindow;

pub struct WindowManager {
    windows: HashMap<WindowId, ManagedWindow>,
    z_order: Vec<WindowId>,
    screen: Size,
}

impl WindowManager {
    pub fn new(screen: Size) -> Self {
        Self {
            windows: HashMap::new(),
            z_order: Vec::new(),
            screen,
        }
    }

    pub fn add_window(
        &mut self,
        id: WindowId,
        title: impl Into<String>,
        position: Point,
        size: Size,
    ) {
        self.windows
            .insert(id, ManagedWindow::new(id, title, position, size));
        if !self.z_order.contains(&id) {
            self.z_order.push(id);
        }
    }

    pub fn remove_window(&mut self, id: WindowId) {
        self.windows.remove(&id);
        self.z_order.retain(|&w| w != id);
    }

    /// Přesune okno na vrchol z-stacku (focus).
    pub fn raise(&mut self, id: WindowId) {
        if self.windows.contains_key(&id) {
            self.z_order.retain(|&w| w != id);
            self.z_order.push(id);
        }
    }

    pub fn move_window(&mut self, id: WindowId, position: Point) {
        if let Some(w) = self.windows.get_mut(&id) {
            w.move_to(position);
        }
    }

    pub fn resize_window(&mut self, id: WindowId, size: Size) {
        if let Some(w) = self.windows.get_mut(&id) {
            w.resize_to(size);
        }
    }

    pub fn minimize_window(&mut self, id: WindowId) {
        if let Some(w) = self.windows.get_mut(&id) {
            w.minimize();
        }
    }

    pub fn restore_window(&mut self, id: WindowId) {
        if let Some(w) = self.windows.get_mut(&id) {
            w.restore();
        }
    }

    pub fn maximize_window(&mut self, id: WindowId) {
        let screen = self.screen;
        if let Some(w) = self.windows.get_mut(&id) {
            w.maximize(screen);
        }
    }

    pub fn snap_window(&mut self, id: WindowId, zone: SnapZone) {
        let screen = self.screen;
        if let Some(w) = self.windows.get_mut(&id) {
            w.snap_to(zone, screen);
        }
    }

    pub fn get(&self, id: WindowId) -> Option<&ManagedWindow> {
        self.windows.get(&id)
    }

    /// Vrátí WindowId v z-pořadí: první = nejníže, poslední = nahoře (focused).
    pub fn z_order(&self) -> &[WindowId] {
        &self.z_order
    }

    pub fn window_count(&self) -> usize {
        self.windows.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn screen() -> Size {
        Size { width: 1920, height: 1080 }
    }

    fn pos(x: f32, y: f32) -> Point {
        Point { x, y }
    }

    fn sz(w: u32, h: u32) -> Size {
        Size { width: w, height: h }
    }

    #[test]
    fn add_window_increases_count() {
        let mut wm = WindowManager::new(screen());
        wm.add_window(WindowId(1), "App", pos(0.0, 0.0), sz(800, 600));
        assert_eq!(wm.window_count(), 1);
        assert!(wm.get(WindowId(1)).is_some());
    }

    #[test]
    fn remove_window_decreases_count_and_clears_z_order() {
        let mut wm = WindowManager::new(screen());
        wm.add_window(WindowId(1), "App", pos(0.0, 0.0), sz(800, 600));
        wm.remove_window(WindowId(1));
        assert_eq!(wm.window_count(), 0);
        assert!(!wm.z_order().contains(&WindowId(1)));
    }

    #[test]
    fn raise_moves_window_to_top_of_z_order() {
        let mut wm = WindowManager::new(screen());
        wm.add_window(WindowId(1), "A", pos(0.0, 0.0), sz(400, 300));
        wm.add_window(WindowId(2), "B", pos(50.0, 50.0), sz(400, 300));
        wm.add_window(WindowId(3), "C", pos(100.0, 100.0), sz(400, 300));
        wm.raise(WindowId(1));
        let z = wm.z_order();
        assert_eq!(z[z.len() - 1], WindowId(1));
        assert_eq!(z.len(), 3);
    }

    #[test]
    fn move_window_changes_position() {
        let mut wm = WindowManager::new(screen());
        wm.add_window(WindowId(1), "App", pos(0.0, 0.0), sz(800, 600));
        wm.move_window(WindowId(1), pos(200.0, 150.0));
        let w = wm.get(WindowId(1)).unwrap();
        assert!((w.position.x - 200.0).abs() < f32::EPSILON);
        assert!((w.position.y - 150.0).abs() < f32::EPSILON);
    }

    #[test]
    fn minimize_and_restore_window() {
        let mut wm = WindowManager::new(screen());
        wm.add_window(WindowId(1), "App", pos(0.0, 0.0), sz(800, 600));
        wm.minimize_window(WindowId(1));
        assert!(!wm.get(WindowId(1)).unwrap().is_visible());
        wm.restore_window(WindowId(1));
        assert!(wm.get(WindowId(1)).unwrap().is_visible());
    }

    #[test]
    fn maximize_window_fills_screen() {
        let mut wm = WindowManager::new(screen());
        wm.add_window(WindowId(1), "App", pos(100.0, 100.0), sz(400, 300));
        wm.maximize_window(WindowId(1));
        let w = wm.get(WindowId(1)).unwrap();
        assert_eq!(w.size.width, 1920);
        assert_eq!(w.size.height, 1080);
    }

    #[test]
    fn snap_window_snaps_to_right_zone() {
        let mut wm = WindowManager::new(screen());
        wm.add_window(WindowId(1), "App", pos(0.0, 0.0), sz(800, 600));
        wm.snap_window(WindowId(1), SnapZone::Right);
        let w = wm.get(WindowId(1)).unwrap();
        assert_eq!(w.size.width, 960);
        assert!((w.position.x - 960.0).abs() < f32::EPSILON);
    }

    #[test]
    fn z_order_reflects_insertion_order() {
        let mut wm = WindowManager::new(screen());
        wm.add_window(WindowId(1), "A", pos(0.0, 0.0), sz(400, 300));
        wm.add_window(WindowId(2), "B", pos(50.0, 50.0), sz(400, 300));
        assert_eq!(wm.z_order(), &[WindowId(1), WindowId(2)]);
    }
}
```

- [ ] **Step 2: Spusť testy**

```bash
cd /Users/vojtechprouza/projekty/AirOS && cargo test -p airwm manager 2>&1 | tail -5
```

Očekávaný výstup: `test result: ok. 8 passed`

- [ ] **Step 3: Commit**

```bash
cd /Users/vojtechprouza/projekty/AirOS
git add crates/airwm/src/manager.rs
git commit -m "feat(airwm): add WindowManager with z-order, move, resize, minimize, maximize, snap"
```

---

## Task 10: WorkspaceManager + celkový build

**Files:**
- Modify: `crates/airwm/src/workspace.rs`
- Modify: `crates/airwm/src/lib.rs`

- [ ] **Step 1: Napiš `workspace.rs`**

```rust
// crates/airwm/src/workspace.rs

use airproto::types::WindowId;

/// Virtuální plocha (workspace/desktop).
#[derive(Debug, Clone)]
pub struct Workspace {
    pub id: u32,
    pub name: String,
    window_ids: Vec<WindowId>,
}

impl Workspace {
    fn new(id: u32, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            window_ids: Vec::new(),
        }
    }

    pub fn window_ids(&self) -> &[WindowId] {
        &self.window_ids
    }

    pub fn contains_window(&self, id: WindowId) -> bool {
        self.window_ids.contains(&id)
    }
}

/// Správce virtuálních ploch.
pub struct WorkspaceManager {
    workspaces: Vec<Workspace>,
    active_idx: usize,
    next_id: u32,
}

impl WorkspaceManager {
    /// Vytvoří manažer s jednou výchozí plochou "Desktop 1".
    pub fn new() -> Self {
        Self {
            workspaces: vec![Workspace::new(0, "Desktop 1")],
            active_idx: 0,
            next_id: 1,
        }
    }

    /// Přidá novou plochu a vrátí její ID.
    pub fn add_workspace(&mut self, name: impl Into<String>) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.workspaces.push(Workspace::new(id, name));
        id
    }

    /// Přepne na plochu dle ID. Vrátí false pokud plocha neexistuje.
    pub fn switch_to(&mut self, id: u32) -> bool {
        if let Some(idx) = self.workspaces.iter().position(|w| w.id == id) {
            self.active_idx = idx;
            true
        } else {
            false
        }
    }

    pub fn active(&self) -> &Workspace {
        &self.workspaces[self.active_idx]
    }

    /// Přiřadí okno na konkrétní plochu (odstraní ho z ostatních ploch).
    pub fn assign_window(&mut self, window_id: WindowId, workspace_id: u32) {
        for ws in &mut self.workspaces {
            ws.window_ids.retain(|&id| id != window_id);
        }
        if let Some(ws) = self.workspaces.iter_mut().find(|w| w.id == workspace_id) {
            ws.window_ids.push(window_id);
        }
    }

    /// Odstraní okno ze všech ploch (při zavření).
    pub fn remove_window(&mut self, window_id: WindowId) {
        for ws in &mut self.workspaces {
            ws.window_ids.retain(|&id| id != window_id);
        }
    }

    /// Vrátí ID plochy, na které okno je, nebo None.
    pub fn workspace_for_window(&self, window_id: WindowId) -> Option<u32> {
        self.workspaces
            .iter()
            .find(|w| w.contains_window(window_id))
            .map(|w| w.id)
    }

    pub fn workspaces(&self) -> &[Workspace] {
        &self.workspaces
    }
}

impl Default for WorkspaceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_has_one_workspace_named_desktop_1() {
        let wm = WorkspaceManager::new();
        assert_eq!(wm.workspaces().len(), 1);
        assert_eq!(wm.active().name, "Desktop 1");
        assert_eq!(wm.active().id, 0);
    }

    #[test]
    fn add_workspace_returns_incrementing_id() {
        let mut wm = WorkspaceManager::new();
        let id1 = wm.add_workspace("Desktop 2");
        let id2 = wm.add_workspace("Desktop 3");
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(wm.workspaces().len(), 3);
    }

    #[test]
    fn switch_to_changes_active_workspace() {
        let mut wm = WorkspaceManager::new();
        let id = wm.add_workspace("Desktop 2");
        let switched = wm.switch_to(id);
        assert!(switched);
        assert_eq!(wm.active().id, id);
    }

    #[test]
    fn switch_to_nonexistent_returns_false() {
        let mut wm = WorkspaceManager::new();
        assert!(!wm.switch_to(99));
        assert_eq!(wm.active().id, 0); // aktivní zůstane beze změny
    }

    #[test]
    fn assign_window_tracks_workspace() {
        let mut wm = WorkspaceManager::new();
        let ws2 = wm.add_workspace("Desktop 2");
        wm.assign_window(WindowId(1), ws2);
        assert_eq!(wm.workspace_for_window(WindowId(1)), Some(ws2));
        assert!(!wm.workspaces()[0].contains_window(WindowId(1)));
    }

    #[test]
    fn remove_window_unassigns_from_all_workspaces() {
        let mut wm = WorkspaceManager::new();
        wm.assign_window(WindowId(1), 0);
        wm.remove_window(WindowId(1));
        assert_eq!(wm.workspace_for_window(WindowId(1)), None);
    }
}
```

- [ ] **Step 2: Aktualizuj `crates/airwm/src/lib.rs`**

```rust
pub mod manager;
pub mod snap;
pub mod window;
pub mod workspace;

pub use manager::WindowManager;
pub use snap::{compute_snap, snap_rect, SnapZone};
pub use window::{ManagedWindow, WindowState};
pub use workspace::{Workspace, WorkspaceManager};
```

- [ ] **Step 3: Spusť testy pro workspace**

```bash
cd /Users/vojtechprouza/projekty/AirOS && cargo test -p airwm workspace 2>&1 | tail -5
```

Očekávaný výstup: `test result: ok. 6 passed`

- [ ] **Step 4: Spusť všechny workspace testy**

```bash
cd /Users/vojtechprouza/projekty/AirOS && cargo test 2>&1 | grep "test result"
```

Očekávaný výstup: všechny crates passing, 0 failing. Celkem 121+ testů (74 existující + 47 nových).

- [ ] **Step 5: Clippy + fmt**

```bash
cd /Users/vojtechprouza/projekty/AirOS && cargo clippy -- -D warnings 2>&1 | tail -5
cargo fmt
```

- [ ] **Step 6: Commit**

```bash
cd /Users/vojtechprouza/projekty/AirOS
git add crates/airwm/src/workspace.rs crates/airwm/src/lib.rs
git commit -m "feat(airwm): add WorkspaceManager; wire up all re-exports in lib.rs"
```

---

## Self-Review Checklist

- [x] **Spec coverage:**
  - AirShell: top bar ✓ (TopBarState s RegisterMenu), dock ✓ (DockState), notifikace ✓ (NotificationQueue), AirBus napojení ✓ (handle_bus_message)
  - AirWM: floating okna ✓ (WindowManager), snap ✓ (SnapZone + compute_snap + ManagedWindow.snap_to), workspaces ✓ (WorkspaceManager), okno anatomy (title bar buttons) — renderování patří do pozdějšího plánu, state model (minimize/maximize) ✓
- [x] **Placeholders:** Žádné TBD/TODO
- [x] **Konzistence typů:**
  - `SnapZone` definováno v `snap.rs` (Task 7), použito v `window.rs` (Task 8), `manager.rs` (Task 9), `lib.rs` (Task 10) — konzistentní
  - `ManagedWindow` definováno v `window.rs` (Task 8), použito v `manager.rs` (Task 9) — konzistentní
  - `ShellState` definováno v `topbar.rs` (Task 4), importováno v `bus_handler.rs` (Task 5) — konzistentní
  - `MenuItem` z `airbus::messages` — použito v `topbar.rs` a `bus_handler.rs` — konzistentní
  - `WindowId`, `Size`, `Point` z `airproto::types` — použito v `window.rs`, `manager.rs`, `workspace.rs` — konzistentní
