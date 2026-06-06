# AirOS Alpine Rootfs Design (Sub-projekt 2)

**Goal:** Sestavit bootovatelný disk image pro QEMU — Alpine Linux rootfs s `aircomp` jako jediným uživatelským procesem.

**Architecture:** Docker-based build pipeline: cross-compile aircomp pro musl, sestavit Alpine Docker image s X11 + Vulkan, exportovat rootfs, vytvořit ext4 disk image uvnitř privilegovaného Docker kontejneru (loop device přístup), spustit QEMU s přímým kernel bootem.

**Scope:** QEMU runtime test. VirtualBox packaging (Sub-projekt 4) je samostatný cyklus.

---

## Co se změní

### Nové soubory

```
scripts/
├── Dockerfile.rootfs      — Alpine image: X11 + Vulkan + aircomp + auto-start
├── build.sh               — orchestrace: cross-compile → Docker build → disk image
└── run-qemu.sh            — QEMU launch command
```

### Změněné soubory

```
Cross.toml                 — změna default-target na x86_64-unknown-linux-musl
.cargo/config.toml         — swap linkeru na x86_64-linux-musl-gcc
.github/workflows/ci.yml   — update target na musl
```

Žádné změny v Rust source kódu.

---

## Detailní design

### Proč musl místo gnu

Alpine Linux používá musl libc. Binárka zkompilovaná pro `x86_64-unknown-linux-gnu` (glibc) na Alpine nespustí bez kompatibilní vrstvy. Řešení: změnit cross-compilation target na `x86_64-unknown-linux-musl` — Rust code se staticky slinkuje s musl, wgpu loaduje Vulkan/X11 přes dlopen (runtime, nezávisí na libc).

### `Cross.toml`

```toml
[build]
default-target = "x86_64-unknown-linux-musl"
```

### `.cargo/config.toml`

```toml
[target.x86_64-unknown-linux-musl]
linker = "x86_64-linux-musl-gcc"
```

Stávající `[target.x86_64-unknown-linux-gnu]` blok zůstane pro případ potřeby, musl blok přibude.

### `crates/aircomp/Cargo.toml`

winit features sekce `[target.'cfg(target_os = "linux")'.dependencies]` platí pro oba Linux targety (gnu i musl) — žádná změna potřeba.

### `.github/workflows/ci.yml`

Změna v `linux-cross` jobu:
```yaml
targets: x86_64-unknown-linux-musl
# ...
run: cross build -p aircomp --target x86_64-unknown-linux-musl
```

### `scripts/Dockerfile.rootfs`

Alpine 3.21 base. Nainstalované balíčky:

**Kernel (pro QEMU boot):**
- `linux-lts` — kernel `vmlinuz-lts` + `initramfs-lts`

**X11 stack:**
- `xorg-server` — Xorg display server
- `xf86-video-fbdev` — fbdev driver pro QEMU VGA (`-vga std`)
- `xinit` — spuštění X session

**winit runtime dependencies (dynamicky loadované):**
- `libx11`, `libxext`, `libxcursor`, `libxi`, `libxrandr` — X11 libs pro winit x11 feature

**wgpu / Vulkan (software rendering):**
- `vulkan-loader` — `libvulkan.so.1`
- `mesa-vulkan-swrast` — lavapipe ICD (`lvp_icd.x86_64.json`), CPU-based Vulkan

**Auto-start konfigurace:**

`/etc/inittab` — auto-login root na tty1:
```
tty1::respawn:/sbin/agetty --autologin root --noclear tty1 linux
```

`/root/.profile` — spustit X při loginu na tty1:
```sh
[ "$(tty)" = "/dev/tty1" ] && exec startx -- -nolisten tcp
```

`/root/.xinitrc` — spustit aircomp:
```sh
#!/bin/sh
xset s off
xset -dpms
exec /usr/local/bin/aircomp
```

`aircomp` binárka se kopíruje z build contextu do `/usr/local/bin/aircomp`.

### Display stack — render chain

```
wgpu render → lavapipe (CPU Vulkan) → xcb/xlib surface → X11 window
    → Xorg fbdev driver → QEMU VGA (-vga std) → QEMU window na macOS
```

lavapipe je pure CPU Vulkan implementace — nepotřebuje GPU hardware. wgpu najde ICD automaticky (`/usr/share/vulkan/icd.d/lvp_icd.x86_64.json`).

### `scripts/build.sh`

Kroky:
1. `cross build --release -p aircomp --target x86_64-unknown-linux-musl`
2. `cp target/x86_64-unknown-linux-musl/release/aircomp scripts/aircomp`
3. `docker build -f scripts/Dockerfile.rootfs -t airos-rootfs scripts/`
4. `docker create --name airos-tmp airos-rootfs`
5. `docker export airos-tmp > /tmp/airos-rootfs.tar && docker rm airos-tmp`
6. Spustit privilegovaný Docker kontejner (Alpine s util-linux) pro vytvoření disk image:
   - `dd` → 512 MB raw image `airos.img`
   - `mkfs.ext4 airos.img`
   - `mount -o loop airos.img /mnt`
   - `tar -xf /airos-rootfs.tar -C /mnt`
   - Zkopírovat `/mnt/boot/vmlinuz-lts` a `/mnt/boot/initramfs-lts` ven
   - `umount /mnt`
7. Artifacts v root projektu: `airos.img`, `vmlinuz-lts`, `initramfs-lts`

Disk image operace běží uvnitř `docker run --privileged` (Linux VM v Docker Desktop) — přístup k loop devices na macOS bez sudo nebo FUSE.

### `scripts/run-qemu.sh`

```bash
#!/bin/sh
qemu-system-x86_64 \
  -kernel vmlinuz-lts \
  -initrd initramfs-lts \
  -append "root=/dev/sda rw console=tty1 quiet" \
  -drive file=airos.img,format=raw \
  -m 1G \
  -smp 2 \
  -vga std \
  -display cocoa
```

Přímý kernel boot — žádný bootloader (GRUB) není potřeba pro QEMU. Bootloader přijde v Sub-projektu 4 (VirtualBox ISO).

### `.gitignore`

Přidat generované artefakty:
```
airos.img
vmlinuz-lts
initramfs-lts
scripts/aircomp
```

---

## Testování

### Build test

```bash
./scripts/build.sh
```

Očekávaný výsledek: soubory `airos.img` (512 MB), `vmlinuz-lts`, `initramfs-lts` v root projektu.

### Runtime test (vizuální)

```bash
./scripts/run-qemu.sh
```

Očekávaný výsledek: QEMU okno → Alpine bootuje → shell chrome viditelný (tmavé pozadí `#141414`, topbar nahoře, dock dole).

---

## Co je mimo scope tohoto sub-projektu

- Wayland display server (X11 stačí pro MVP)
- Síťování v Alpine
- Keyboard/mouse input handling v aircomp
- GRUB bootloader (Sub-projekt 4)
- VirtualBox packaging (Sub-projekt 4)
- Apps v oknech — airterm, airfiles (Sub-projekt 3)
