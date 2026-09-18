#!/usr/bin/env bash

IMAGE="${IMAGE:-HeineOS.img}"
BIOS="${BIOS:-RELEASEX64_OVMF.fd}"
QEMU_AUDIO_DEVICE="${QEMU_AUDIO_DEVICE:-pa}"

if [ ! -f "$IMAGE" ]; then
    echo "❌ Error: Image file '$IMAGE' not found!"
    exit 1
fi

if [ ! -f "$BIOS" ]; then
    echo "❌ Error: BIOS file '$BIOS' not found!"
    exit 1
fi

# Prefer KVM acceleration if available, otherwise fall back to TCG.
ACCEL_ARGS=()
if [ -w /dev/kvm ]; then
    ACCEL_ARGS=( -accel kvm )
else
    echo "⚠️  KVM unavailable, falling back to TCG emulation."
    ACCEL_ARGS=( -accel tcg,thread=multi )
fi

qemu-system-x86_64 \
  -machine q35,pcspk-audiodev=audio0 \
  -m 512M \
  -bios ${BIOS} \
  -boot d \
  -vga std \
  -rtc base=localtime \
  -serial stdio \
  -drive driver=raw,if=none,id=boot,file.filename="${IMAGE}" \
  -device ide-hd,drive=boot \
  -audiodev id=audio0,driver="${QEMU_AUDIO_DEVICE}" \
  -nic model=rtl8139,id=rtl8139,hostfwd=udp::1797-:1797,hostfwd=tcp::1797-:1797 \
  -object filter-dump,id=filter1,netdev=rtl8139,file=rtl8139.dump \
  "${ACCEL_ARGS[@]}"