# AirOS Linux Port Design (Sub-projekt 1)

**Goal:** Sestavit `aircomp` pro `x86_64-unknown-linux-gnu` pomocí `cross` — produkovat spustitelnou Linux ELF binárku bez změn logiky.

**Architecture:** Použít `cross` (Docker-based cross-compilation tool) pro build. wgpu a winit jsou cross-platform — na Linuxu automaticky zvolí Vulkan/llvmpipe a Wayland/X11. Žádné změny Rust kódu, jen build konfigurace.

**Scope:** Jen build infrastruktura. Runtime test je manuální (QEMU). Sub-projekty 2-4 (Alpine rootfs, apps v oknech, VirtualBox ISO) jsou samostatné spec/plán cykly.

---

## Co se změní

### Soubory

```
AirOS/
├── Cross.toml                        CREATE — cross build konfigurace
├── .cargo/
│   └── config.toml                   CREATE — target-specific linker + env
├── crates/aircomp/Cargo.toml         MODIFY — přidat wayland+x11 features pro winit
└── .github/workflows/ci.yml          MODIFY — přidat Linux cross-build job
```

Žádné změny v Rust source kódu.

---

## Detailní design

### `Cross.toml`

`cross` Docker image pro `x86_64-unknown-linux-gnu` obsahuje Vulkan headers a Wayland dev knihovny. Žádné extra konfigurace nejsou potřeba — výchozí image stačí.

```toml
[build]
default-target = "x86_64-unknown-linux-gnu"
```

### `.cargo/config.toml`

Specifikuje linker pro Linux target (potřebný při native cross-kompilaci bez `cross`). S `cross` to Docker image řeší automaticky, ale config je potřeba pro lokální build i CI.

```toml
[target.x86_64-unknown-linux-gnu]
linker = "x86_64-linux-gnu-gcc"
```

### `crates/aircomp/Cargo.toml` — winit features

winit 0.30 na Linuxu potřebuje explicitní feature flag pro Wayland a/nebo X11:

```toml
[target.'cfg(target_os = "linux")'.dependencies]
winit = { version = "0.30", features = ["wayland", "x11"] }
```

Existující `winit = "0.30"` (bez features) zůstane pro macOS. Linux-specific sekce přidá Wayland+X11 support.

**Alternativa** pokud target-specific deps způsobí problém: přidat features globálně (`winit = { version = "0.30", features = ["wayland", "x11"] }`) — macOS je ignoruje.

### `.github/workflows/ci.yml` — Linux cross-build job

Přidat nový job vedle existujícího macOS jobu:

```yaml
linux-cross:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        targets: x86_64-unknown-linux-gnu
    - name: Install cross
      run: cargo install cross --git https://github.com/cross-rs/cross
    - name: Cache
      uses: Swatinem/rust-cache@v2
    - name: Cross build aircomp
      run: cross build -p aircomp --target x86_64-unknown-linux-gnu
```

Tento job ověří že Linux build prochází na každý push. Testy se nespouštějí (vyžadují display server).

---

## Testování

### Build test (automatický — CI)

```bash
cross build -p aircomp --target x86_64-unknown-linux-gnu
```

Očekávaný výsledek: `Finished` + `target/x86_64-unknown-linux-gnu/debug/aircomp` ELF binárka.

### Runtime test (manuální — QEMU)

```bash
# Nainstalovat QEMU
brew install qemu

# Stáhnout Alpine Linux ISO (x86_64, ~60MB)
# https://dl-cdn.alpinelinux.org/alpine/v3.21/releases/x86_64/alpine-standard-3.21.0-x86_64.iso

# Spustit QEMU s Alpine
qemu-system-x86_64 \
  -cdrom alpine-standard-3.21.0-x86_64.iso \
  -m 512M \
  -vga std \
  -display cocoa

# V Alpine: nainstalovat Xorg, zkopírovat binárku, spustit
# (detaily v implementačním plánu)
```

Očekávaný výsledek: shell chrome (tmavé pozadí + topbar + dock) viditelný v QEMU okně.

---

## Co je mimo scope tohoto sub-projektu

- Alpine rootfs image (Sub-projekt 2)
- Apps v oknech — airterm, airfiles (Sub-projekt 3)
- VirtualBox ISO packaging (Sub-projekt 4)
- Změny Rust logiky specifické pro Linux
- Audio, networking, filesystem mount v Linuxu
