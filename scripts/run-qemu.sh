#!/bin/sh
set -e

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

# Ověřit artefakty
for f in airos.img vmlinuz-lts initramfs-lts; do
    if [ ! -f "$ROOT_DIR/$f" ]; then
        echo "ERROR: $f nenalezen. Spusť nejdřív: ./scripts/build.sh"
        exit 1
    fi
done

# Ověřit dostupnost QEMU
command -v qemu-system-x86_64 >/dev/null 2>&1 || { echo "ERROR: qemu-system-x86_64 není nainstalován. Spusť: brew install qemu"; exit 1; }

echo "==> Spouštím AirOS v QEMU..."
qemu-system-x86_64 \
    -kernel "$ROOT_DIR/vmlinuz-lts" \
    -initrd "$ROOT_DIR/initramfs-lts" \
    -append "root=/dev/sda rw console=tty1 quiet" \
    -drive file="$ROOT_DIR/airos.img",format=raw \
    -m 1G \
    -smp 2 \
    -vga std \
    -display cocoa  # macOS — zmeň na -display gtk pro Linux
