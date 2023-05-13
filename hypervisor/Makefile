ARCH ?= aarch64
MODE ?= release
LOG ?= warn
SMP ?= 2
GUEST ?= nimbos

GUEST_IMG_NIMBOS ?= ../bin/nimbos.bin
GUEST_IMG_1 ?= ../bin/arceos-httpserver-single.bin
GUEST_IMG_2 ?= ../bin/arceos-shell.bin
GUEST_IMG_3 ?= ../bin/arceos-parallel-dual.bin
DISK_IMG ?= ../bin/disk.img

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
DUMPDTB ?= n


ifeq ($(shell uname), Darwin)
	GDB := aarch64-none-elf-gdb
else
	GDB := gdb-multiarch
endif

qemu := qemu-system-$(ARCH)
qemu_args := -nographic -m 1G

ifeq ($(ARCH), aarch64)
	qemu_args += \
		-cpu cortex-a72 \
		-machine type=virt,virtualization=on,iommu=smmuv3 \
		-kernel $(target_bin) \
		-smp $(SMP)
endif

ifeq ($(GUEST), nimbos)
	qemu_args += -device loader,addr=0x4001000,file=$(GUEST_IMG_NIMBOS),force-raw=on
else 
	qemu_args += -device loader,addr=0x4001000,file=$(GUEST_IMG_1),force-raw=on
	qemu_args += -device loader,addr=0x4101000,file=$(GUEST_IMG_2),force-raw=on
	qemu_args += -device loader,addr=0x4201000,file=$(GUEST_IMG_3),force-raw=on
endif

ifeq ($(NET), y)
	qemu_args += \
		-device virtio-net-device,netdev=net0 \
		-netdev user,id=net0,hostfwd=tcp::5555-:5555
endif

ifeq ($(FS), y)
	qemu_args += \
		-device virtio-blk-device,drive=disk0 \
		-drive id=disk0,if=none,format=raw,file=$(DISK_IMG)
endif

# qemu_args += \
#     --trace "virtio_blk_handle_request" \
#     --trace "virtio_blk_handle_read" \
# 	--trace "virtio_blk_handle_write" \
# 	--trace "virtio_notify" \
# 	--trace "virtio_queue_notify" \
# 	--trace "virtio_blk_data_plane_start" \
# 	--trace "virtio_blk_req_complete"

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