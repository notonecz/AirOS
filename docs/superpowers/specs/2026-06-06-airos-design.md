# AirOS — Design Spec

**Datum:** 2026-06-06
**Verze:** 1.0
**Stav:** Schváleno

---

## Přehled

AirOS je vlastní operační systém postavený na Darwin (XNU) kernelu. Cílí na běžné uživatele a tvůrce jako přístupná, vizuálně výrazná alternativa. Userspace je implementován výhradně v Rustu. Podporuje ARM64 (Apple Silicon) i x86_64 jako universal binary.

---

## Vrstvová architektura

```
┌─────────────────────────────────────────────┐
│           APLIKACE (AirFiles, AirTerm...)    │
├─────────────────────────────────────────────┤
│        AIRKIT — App Framework / SDK          │
├──────────────────┬──────────────────────────┤
│   AIRSHELL       │   AIRWM                  │
│   (dock, bar,    │   (window manager,       │
│    launcher,     │    workspaces,           │
│    notifications)│    animace)              │
├──────────────────┴──────────────────────────┤
│         AIRCOMP — Rust Compositor            │
│   (display server, IPC, GPU rendering)       │
├─────────────────────────────────────────────┤
│         DARWIN KERNEL (XNU)                  │
│   process mgmt · FS · networking · IOKit     │
└─────────────────────────────────────────────┘
```

Každá vrstva je samostatný Rust workspace crate komunikující přes jasně definované IPC rozhraní. Darwin kernel se nedotýká.

---

## AirComp — Compositor

AirComp nahrazuje macOS WindowServer. Běží jako privilegovaný proces s přímým přístupem k display hardware přes IOKit (Darwin open source).

**Zodpovědnosti:**
- Inicializace display(ů) přes IOKit + wgpu surface
- Přijímá okna od aplikací jako textury / render buffers
- Compositor loop: compositing oken → frame → display
- Správa vstupu (klávesnice, myš, trackpad) přes IOHIDManager
- IPC server — Unix domain sockets, binární protokol (AirProto)

**IPC protokol (AirProto):**
```
Client (app)  ←── Unix socket ──→  AirComp
  window_create(id, size)
  window_destroy(id)
  buffer_commit(id, pixels)
  window_resize(id, new_size)
  ←── event(KeyPress | MouseMove | WindowFocus | WindowClose)
```

**Render pipeline:**
```
App buffer → wgpu texture upload → compositor shader → scanout
```

Každé okno je GPU textura. Compositor shader skládá vrstvy, aplikuje průhlednost, stíny a přechody. Cílový framerate: 60/120 fps dle refresh rate displeje.

**GPU stack:** `wgpu` jako cross-platform abstrakce — Metal backend na ARM64, Vulkan/OpenGL na x86_64. Compositor kód je identický pro obě platformy.

---

## AirShell — Desktop prostředí (macOS-inspired)

- **Top bar:** hodiny uprostřed, systémové ikony vpravo (WiFi, baterie, zvuk, tray), menu bar vlevo
- **Dock:** spodek obrazovky, spuštěné + pinned aplikace, magnification efekt při hoveru
- **Launcher:** Spotlight-like — kbd shortcut → search bar uprostřed obrazovky, výsledky v reálném čase
- **Notifikace:** slide-in z pravého horního rohu, notification center jako panel

Komunikuje s AirComp přes AirProto a s aplikacemi přes AirBus.

---

## AirWM — Správa oken (Windows-inspired)

- **Floating výchozí:** překrývající se okna, volné přesouvání a resize
- **Snap:** přetáhni okno na kraj/roh → snap do půlky nebo čtvrtiny obrazovky
- **Taskbar preview:** hover nad ikonou v docku → thumbnail náhled okna
- **Workspaces:** Mission Control-like přehled všech virtuálních ploch
- **Okno anatomie:** title bar s ikonou vlevo, tlačítka (minimalizovat, maximalizovat, zavřít) vpravo

```
┌──────────────────────────────────────┐
│  [ikona] Název aplikace   _ □ ✕      │
├──────────────────────────────────────┤
│        app content                   │
└──────────────────────────────────────┘
```

Title bar a dekorace kreslí AirWM, obsah kreslí aplikace.

---

## Core aplikace (MVP)

Každá aplikace je samostatný Rust binary, UI staví přes `airkit-ui`.

### AirFiles — správce souborů
- Dvoupanelový layout (volitelný) nebo single panel s breadcrumb navigací
- Sidebar: oblíbené, zařízení, tagy
- Náhled souborů (obrázky, text, PDF)
- Drag & drop mezi okny

### AirTerm — terminál
- Plnohodnotný terminál (PTY přes Darwin `posix_openpt`)
- Tabs + split panes
- GPU-accelerated text rendering (`cosmic-text` crate)
- Konfigurovatelný shell (zsh výchozí na Darwinu)

### AirSettings — nastavení systému
- Sidebar s kategoriemi: Displej, Zvuk, Síť, Uživatelé, Vzhled, O systému
- Appearance sekce: theme tokeny (barvy, accent color, tmavý/světlý režim)
- Přímé API na systémové démony přes IPC

### AirSetup — first-run průvodce
- Wizard při prvním spuštění: jazyk, uživatel, síť, vzhled
- Spustí se pouze jednou, pak se nespustí znovu

---

## App model & IPC

Každá aplikace je standardní Darwin process komunikující přes dva kanály:

```
App process
  ├── AirProto socket  →  AirComp   (okna, rendering, vstup)
  └── AirBus socket    →  AirShell  (notifikace, menu bar položky, dock badge)
```

**AirBus zprávy (MessagePack formát):**
- `notify(title, body, icon)` — odeslání notifikace
- `set_dock_badge(count)` — číslo na dock ikoně
- `register_menu(items)` — položky v top bar menu

**App bundle formát:**
```
MyApp.air/
├── Info.toml          # název, verze, ikona, oprávnění
├── bin/myapp          # ARM64 + x86_64 universal binary
└── assets/            # ikony, fonty, resources
```

Sandbox není součástí MVP — přidá se v pozdější verzi.

---

## Build & Distribution

**Cargo workspace:**
```
airos/
├── Cargo.toml
├── crates/
│   ├── aircomp/
│   ├── airshell/
│   ├── airwm/
│   ├── airkit-ui/
│   ├── airkit-theme/
│   ├── airproto/
│   └── airbus/
└── apps/
    ├── airfiles/
    ├── airterm/
    ├── airsettings/
    └── airsetup/
```

**Multiarch build:**
```bash
cargo build --target aarch64-apple-darwin
cargo build --target x86_64-apple-darwin
lipo -create ...   # universal binary
```

**Distribuce:**
- ISO image s Darwin kernelem + AirOS userspace
- Instalátor jako první spuštěná aplikace (před AirSetup)
- Atomické aktualizace: nová verze se připraví vedle, swap při restartu

**CI/CD:** GitHub Actions — build obou architektur, unit testy, integrace spouštěná ve QEMU pro x86_64.

---

## Vizuální systém

Centrální `airkit-theme` crate definuje theme tokeny: barvy, accent color, border radius, blur, stíny, spacing. Všechny komponenty v `airkit-ui` konzumují tokeny — tmavý/světlý režim funguje záměnou token setu, ne podmíněnou logikou v komponentách.

---

## Co není v MVP

- App sandbox / permissions model
- App store / package manager pro třetí strany
- Accessibility (screen reader, zvětšení)
- Lokalizace (pouze čeština/angličtina v MVP)
- Systémové démony (spotlight indexer, Time Machine-like backup)
