ARCH ?= aarch64
MODE ?= release
LOG ?= warn
SMP ?= 1
GUEST ?= nimbos

GUEST_IMG_NIMBOS ?= ../bin/nimbos.bin
GUEST_IMG_1 ?= ../bin/arceos-helloworld.bin
GUEST_IMG_2 ?= ../bin/arceos-httpserver-single.bin

export ARCH
export MODE
export LOG

target_elf := target/$(ARCH)/$(MODE)/hypervisor
target_bin := $(target_elf).bin

build_args := --target $(ARCH).json -Zbuild-std=core,alloc -Zbuild-std-features=compiler-builtins-mem
ifeq ($(MODE), release)
  build_args += --release
endif

OBJDUMP := rust-objdump -d --print-imm-hex --x86-asm-syntax=intel
OBJCOPY := rust-objcopy --binary-architecture=$(ARCH)

FS ?= n
NET ?= n
GRAPHIC ?= n


ifeq ($(shell uname), Darwin)
	GDB := aarch64-none-elf-gdb
else
	GDB := gdb-multiarch
endif

qemu := qemu-system-$(ARCH)
qemu_args := -nographic -m 512M

ifeq ($(ARCH), aarch64)
	qemu_args += \
		-cpu cortex-a72 \
		-machine type=virt,virtualization=on \
		-kernel $(target_bin) \
		-smp $(SMP)
endif

ifeq ($(GUEST), nimbos)
	qemu_args += -device loader,addr=0x4001000,file=$(GUEST_IMG_NIMBOS),force-raw=on
else 
	qemu_args += -device loader,addr=0x4001000,file=$(GUEST_IMG_1),force-raw=on
	qemu_args += -device loader,addr=0x4101000,file=$(GUEST_IMG_2),force-raw=on
endif

ifeq ($(NET), y)
	qemu_args += \
		-device virtio-net-device,netdev=net0 \
		-netdev user,id=net0,hostfwd=tcp::5555-:5555
endif

build: $(target_bin)

$(target_bin): elf
	@$(OBJCOPY) $(target_elf) --strip-all -O binary $@

elf:
	@echo Arch = $(ARCH)
	cargo build $(build_args)

clean:
	cargo clean

clippy:
	cargo clippy $(build_args)

disasm:
	@$(OBJDUMP) $(target_elf) | less

run: build justrun

justrun:
	$(qemu) $(qemu_args)

debug: build
	$(qemu) $(qemu_args) -s -S &
	sleep 1
	$(GDB) $(target_elf) -ex 'target remote localhost:1234'

trace: build
	$(qemu) $(qemu_args) -d in_asm > trace.asm

.PHONY: build elf clean clippy disasm run justrun debug trace