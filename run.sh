#!/usr/bin/env bash

IMAGE="${IMAGE:-HeineOS.img}"
BIOS="${BIOS:-RELEASEX64_OVMF.fd}"
QEMU_AUDIO_DEVICE="${QEMU_AUDIO_DEVICE:-pa}"

# For the "pa" backend we pass the server socket explicitly. Without it QEMU stats
# "$XDG_RUNTIME_DIR/pulse/pid" before connecting and aborts with
# "could not stat pidfile" when that file is missing. A pipewire-pulse server only
# writes that pidfile on startup (it is socket activated), so the first QEMU launch
# of a fresh login session would fail. See QEMU's audio/paaudio.c.
XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
QEMU_AUDIO_SERVER="${PULSE_SERVER:-unix:${XDG_RUNTIME_DIR}/pulse/native}"
if [ "$QEMU_AUDIO_DEVICE" = "pa" ]; then
  QEMU_AUDIO_OPTS="id=audio0,driver=pa,server=${QEMU_AUDIO_SERVER}"
else
  QEMU_AUDIO_OPTS="id=audio0,driver=${QEMU_AUDIO_DEVICE}"
fi

if [ ! -f "$IMAGE" ]; then
    echo "❌ Error: Image file '$IMAGE' not found!"
    exit 1
fi

if [ ! -f "$BIOS" ]; then
    echo "❌ Error: BIOS file '$BIOS' not found!"
    exit 1
fi

# Prefer KVM acceleration if available, otherwise fall back to TCG.
# "-cpu host" is only valid with hardware acceleration; under TCG it is rejected,
# so keep the emulated default there.
ACCEL_ARGS=()
CPU_ARGS=()
if [ -w /dev/kvm ]; then
    ACCEL_ARGS=( -accel kvm )
    CPU_ARGS=( -cpu host )
else
    echo "⚠️  KVM unavailable, falling back to TCG emulation."
    ACCEL_ARGS=( -accel tcg,thread=multi )
    CPU_ARGS=( -cpu qemu64 )
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
  -audiodev "${QEMU_AUDIO_OPTS}" \
  -nic model=rtl8139,id=rtl8139,hostfwd=udp::1797-:1797,hostfwd=tcp::1797-:1797 \
  -object filter-dump,id=filter1,netdev=rtl8139,file=rtl8139.dump \
  "${ACCEL_ARGS[@]}" \
  "${CPU_ARGS[@]}"
