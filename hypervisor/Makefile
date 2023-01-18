ARCH ?= aarch64
MODE ?= release
LOG ?= warn

qemu := qemu-system-$(ARCH)
qemu-args := -nographic -m 128M

ifeq ($(ARCH), aarch64)
	ACCEL ?= off
  PLATFORM ?= qemu-virt-arm
endif

export ARCH
export PLATFORM
export MODE
export LOG

qemu := qemu-system-$(ARCH)
qemu-args := -nographic -m 128M