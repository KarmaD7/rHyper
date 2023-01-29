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

target_elf := target/$(ARCH)/$(MODE)/rvm-hypervisor
target_bin := $(target_elf).bin

build_args := --target $(ARCH).json -Zbuild-std=core,alloc -Zbuild-std-features=compiler-builtins-mem
ifeq ($(MODE), release)
  build_args += --release
endif

qemu := qemu-system-$(ARCH)
qemu_args := -nographic -m 128M

build: $(target_bin)

$(target_bin): elf
	@echo 0 > /dev/null

elf:
	@echo Arch = $(ARCH)
	cargo build $(build_args)

run:
	build justrun

justrun:
	$(qemu) $(qemu-args)

clean:
	cargo clean
