#!/bin/sh
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# Ověřit dostupnost nástrojů
command -v cross >/dev/null 2>&1 || { echo "ERROR: 'cross' není nainstalován. Spusť: cargo install cross --git https://github.com/cross-rs/cross --rev 29d00c78"; exit 1; }
command -v docker >/dev/null 2>&1 || { echo "ERROR: Docker není spuštěn nebo nainstalován."; exit 1; }

echo "==> [1/5] Cross-compile aircomp (x86_64-unknown-linux-musl, release)..."
cd "$ROOT_DIR"
cross build --release -p aircomp --target x86_64-unknown-linux-musl

echo "==> [2/5] Kopírování binárky do Docker build contextu..."
cp target/x86_64-unknown-linux-musl/release/aircomp scripts/aircomp

echo "==> [3/5] Sestavení Alpine Docker image..."
docker build -f scripts/Dockerfile.rootfs -t airos-rootfs scripts/
rm -f scripts/aircomp

echo "==> [4/5] Export rootfs tarball..."
trap 'docker rm -f airos-tmp 2>/dev/null || true' EXIT
docker create --name airos-tmp airos-rootfs
docker export airos-tmp > "$ROOT_DIR/airos-rootfs.tar"
docker rm airos-tmp
trap - EXIT

echo "==> [5/5] Vytvoření ext4 disk image (512 MB) uvnitř Docker..."
# --privileged dává přístup k loop devices uvnitř Docker Linux VM
docker run --privileged --rm \
    -v "$ROOT_DIR/airos-rootfs.tar":/rootfs.tar:ro \
    -v "$ROOT_DIR":/output \
    alpine:3.21 sh -c '
        apk add --no-cache e2fsprogs util-linux >/dev/null 2>&1
        dd if=/dev/zero of=/output/airos.img bs=1M count=512 2>/dev/null
        mkfs.ext4 -q /output/airos.img
        mkdir -p /mnt
        mount -o loop /output/airos.img /mnt
        tar -xf /rootfs.tar -C /mnt || true
        cp /mnt/boot/vmlinuz-lts /output/vmlinuz-lts
        cp /mnt/boot/initramfs-lts /output/initramfs-lts
        umount /mnt
    '

rm -f "$ROOT_DIR/airos-rootfs.tar"

echo ""
echo "==> Hotovo! Artefakty:"
echo "    $(du -sh "$ROOT_DIR/airos.img")"
echo "    $ROOT_DIR/vmlinuz-lts"
echo "    $ROOT_DIR/initramfs-lts"
echo ""
echo "Spustit: ./scripts/run-qemu.sh"
