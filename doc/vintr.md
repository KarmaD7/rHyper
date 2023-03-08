ARM 的 GIC 也由 Distributer 和 Redistributor 两部分构成，其中 Distributer 类似于 I/O APIC，其发挥的作用顾名思义，就是根据CPU的配置，将到来的中断源派发到 CPU 对应的Redistributor。Redistributor 类似于Local APIC，它会将Distributer派发来的中断送到其连接的CPU。

这种对中断源的划分是从GIC输入的角度进行的，而中断经过GIC(输出)到达CPU之后，则只有IRQ(Interrupt Request)和FIQ(Fast Interrupt Request)两种，其转换规则有一些复杂，在此就不展开了。

Distributor
CPU interfaces
The Distributor block performs interrupt prioritization and distribution to the CPU interface blocks that connect to the processors in the system.
The Distributor block registers are identified by the GICD_ prefix.
Each CPU interface block performs priority masking and preemption handling for a connected processor in the system.
CPU interface block registers are identified by the GICC_ prefix.
When describing a GIC that includes the GIC Virtualization Extensions, a CPU interface is sometimes called a physical CPU interface, to avoid possible confusion with a virtual CPU interface.
Virtual CPU interfaces
The GIC Virtualization Extensions add a virtual CPU interface for each processor in the system. Each virtual CPU interface is partitioned into the following blocks:
Virtual interface control
The main component of the virtual interface control block is the GIC virtual interface control registers, that include a list of active and pending virtual interrupts for the current virtual machine on the connected processor. Typically, these registers are managed by the hypervisor that is running on that processor.
Virtual interface control block registers are identified by the GICH_ prefix.
Virtual CPU interface
Each virtual CPU interface block provides physical signaling of virtual interrupts to the connected processor. The ARM processor Virtualization Extensions signal these interrupts to the current virtual machine on that processor. The GIC virtual CPU interface registers, accessed by the virtual machine, provide interrupt control and status information for the virtual interrupts. The format of these registers is similar to the format of the physical CPU interface registers.
Virtual CPU interface block registers are identified by the GICV_ prefix.

static const MemMapEntry base_memmap[] = {
    /* Space up to 0x8000000 is reserved for a boot ROM */
    [VIRT_FLASH] =              {          0, 0x08000000 },
    [VIRT_CPUPERIPHS] =         { 0x08000000, 0x00020000 },
    /* GIC distributor and CPU interfaces sit inside the CPU peripheral space */
    [VIRT_GIC_DIST] =           { 0x08000000, 0x00010000 },
    [VIRT_GIC_CPU] =            { 0x08010000, 0x00010000 },
    [VIRT_GIC_V2M] =            { 0x08020000, 0x00001000 },
    [VIRT_GIC_HYP] =            { 0x08030000, 0x00010000 },
    [VIRT_GIC_VCPU] =           { 0x08040000, 0x00010000 },
    /* The space in between here is reserved for GICv3 CPU/vCPU/HYP */
    [VIRT_GIC_ITS] =            { 0x08080000, 0x00020000 },
    /* This redistributor space allows up to 2*64kB*123 CPUs */
    [VIRT_GIC_REDIST] =         { 0x080A0000, 0x00F60000 },
    [VIRT_UART] =               { 0x09000000, 0x00001000 },
    [VIRT_RTC] =                { 0x09010000, 0x00001000 },
    [VIRT_FW_CFG] =             { 0x09020000, 0x00000018 },
    [VIRT_GPIO] =               { 0x09030000, 0x00001000 },
    [VIRT_SECURE_UART] =        { 0x09040000, 0x00001000 },
    [VIRT_SMMU] =               { 0x09050000, 0x00020000 },
    [VIRT_PCDIMM_ACPI] =        { 0x09070000, MEMORY_HOTPLUG_IO_LEN },
    [VIRT_ACPI_GED] =           { 0x09080000, ACPI_GED_EVT_SEL_LEN },
    [VIRT_NVDIMM_ACPI] =        { 0x09090000, NVDIMM_ACPI_IO_LEN},
    [VIRT_MMIO] =               { 0x0a000000, 0x00000200 },
    /* ...repeating for a total of NUM_VIRTIO_TRANSPORTS, each of that size */
    [VIRT_PLATFORM_BUS] =       { 0x0c000000, 0x02000000 },
    [VIRT_SECURE_MEM] =         { 0x0e000000, 0x01000000 },
    [VIRT_PCIE_MMIO] =          { 0x10000000, 0x2eff0000 },
    [VIRT_PCIE_PIO] =           { 0x3eff0000, 0x00010000 },
    [VIRT_PCIE_ECAM] =          { 0x3f000000, 0x01000000 },
    /* Actual RAM size depends on initial RAM and device memory settings */
    [VIRT_MEM] =                { GiB, LEGACY_RAMLIMIT_BYTES },
};