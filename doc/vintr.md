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

GIC中断源的编号从0开始，0到15是SGI(Software Generated Interrupt)，所谓"software generated"，就是指由CPU直接写对应的寄存器触发中断，因而这种中断不是由硬件产生的，而是由软件主动产生的。这种特殊的中断主要用于核间通信，类似于x86中的IPI(Inter-Processor Interrupt)。

编号16到31是PPI(Private Peripheral Interrupt)，所谓"private"，是指这个中断为CPU私有/专用，那什么中断会有这种特性呢？比如通用定时器中断啊，温度传感器中断啊。

SGI和PPI都是每个CPU各有一份，不同CPU的同一种SGI/PPI共享同一个编号，因而它们都属于"banked"形式的中断源。

与专有的PPI相对应的就是所有CPU全局共享的SPI(Shared Peripheral Interrupt)，编号从32到1019。

Any ARM processor implementation that includes the Virtualization Extensions must also include the Security Extensions. Such a processor is usually implemented with a GIC that implements both the GIC Security Extensions and GIC Virtualization Extensions. The examples in this chapter only describe such an implementation, for which:
— Group 0 physical interrupts are Secure interrupts
— Group 1 physical interrupts are Non-secure interrupts.
— the hypervisor performs the initial processing of all physical IRQs, virtualizing them as required as virtual IRQs or virtual FIQs
— Secure Monitor mode performs the initial processing of all physical FIQs.

## Interrupt States

**Inactive **

**Pending**

**Active
 Active and pending**

An interrupt that is not active or pending.

An interrupt from a source to the GIC that is recognized as asserted in hardware, or generated by software, and is waiting to be serviced by a target processor.

An interrupt from a source to the GIC that has been acknowledged by a processor, and is being serviced but has not completed.

**spurious**

A processor is servicing the interrupt and the GIC has a pending interrupt from the same source.

1. It is possible that an interrupt that the GIC has signaled to a processor is no longer required. If this happens, when the processor acknowledges the interrupt, the GIC returns a special Interrupt ID that identifies the interrupt as a *spurious interrupt*.

![截屏2023-03-08 16.22.40](/Users/dsf/Library/Application Support/typora-user-images/截屏2023-03-08 16.22.40.png)

The Distributor provides a programming interface for:

- Globally enabling the forwarding of interrupts to the CPU interfaces.

- Enabling or disabling each interrupt.

- Setting the priority level of each interrupt.

- Setting the target processor list of each interrupt.

- Setting each peripheral interrupt to be level-sensitive or edge-triggered.

- Setting each interrupt as either Group 0 or Group 1.

  **Note**

  For GICv1, setting interrupts as Group 0 or Group 1 is possible only when the implementation includes the GIC Security Extensions.

- Forwarding an SGI to one or more target processors.

- Each connected processor issues an SGI by writing to the GICD_SGIR in the Distributor. Each write can generate SGIs with the same ID that target multiple processors.

Each CPU interface block provides the interface for a processor that is connected to the GIC. Each CPU interface provides a programming interface for:

- enabling the signaling of interrupt requests to the processor

- acknowledging an interrupt

- indicating completion of the processing of an interrupt

- setting an interrupt priority mask for the processor

- defining the preemption policy for the processor

- determining the highest priority pending interrupt for the processor.'

  

At any time, the connected processor can read the priority of its highest priority active interrupt from its GICC_HPPIR, a CPU interface register.



In a GICv2 implementation, the GICC_CTLR.EOImode bit determines whether:

- the two stages happen together, when the processor writes to the CPU interface End of Interrupt register
- the two stages are separated, so that:
  - —  priority drop happens when the processor writes to the CPU interface End of Interrupt register
  - —  interrupt deactivation happens later, when the processor writes to the CPU interface Deactivate Interrupt register.



1. In a GICv2 implementation, the GICC_CTLR.EOImode bit determines whether:
   - the two stages happen together, when the processor writes to the CPU interface End of Interrupt register
   - the two stages are separated, so that:
     - —  priority drop happens when the processor writes to the CPU interface End of Interrupt register
     - —  interrupt deactivation happens later, when the processor writes to the CPU interface Deactivate Interrupt register.



1. Software discovers the interrupts that are supported by:
2. 2. 3.

Reading the GICD_TYPER. The GICD_TYPER.ITLinesNumber field identifies the number of implemented GICD_ISENABLERns, and therefore the maximum number of SPIs that might be supported.

Writing to the GICD_CTLR to disable forwarding of interrupts from the distributor to the CPU interfaces. For more information, see *Enabling and disabling the Distributor and CPU interfaces* on page 4-77.

For each implemented GICD_ISENABLERn, starting with GICD_ISENABLER0:

• •

Writing 0xFFFFFFFF to the GICD_ISENABLERn.

Reading the value of the GICD_ISENABLERn. Bits that read as 1 correspond to supported interrupt IDs.



Software uses the GICD_ICENABLERns to discover the interrupts that are permanently enabled. For each implemented GICD_ICENABLERn, starting with GICD_ICENABLER0, software:

1. Writes 0xFFFFFFFF to the GICD_ICENABLERn. This disables all interrupts that can be disabled.

2. Reads the value of the GICD_ICENABLERn. Bits that read as 1 correspond to interrupts that are

   permanently enabled.

3. Writes 1 to any GICD_ISENABLERn bits corresponding to interrupts that must be re-enabled.





The GIC implements the same number of GICD_ISENABLERns and GICD_ICENABLERns.
 When software has completed its discovery, it typically writes to the GICD_CTLR to re-enable forwarding of interrupts from the Distributor to the CPU interfaces.
 If the GIC implements the GIC Security Extensions, software can use Secure accesses to discover all the supported interrupt IDs, see *The effect of interrupt grouping on interrupt handling* on page 3-48 for more information.

1. The GIC determines whether each interrupt is enabled. An interrupt that is not enabled has no effect on the GIC.
2. For each enabled interrupt that is pending, the Distributor determines the targeted processor or processors.
3. For each processor, the Distributor determines the highest priority pending interrupt, based on the priority information it holds for each interrupt, and forwards the interrupt to the targeted CPU interfaces.
4. If the distributor is forwarding an interrupt request to a CPU interface, the CPU interface determines whether the interrupt has *Sufficient priority* to be signaled to the processor. If the interrupt has sufficient priority, the GIC signals an interrupt request to the processor.
5. When a processor takes the interrupt exception, it reads the GICC_IAR of its CPU interface to acknowledge the interrupt. This read returns an Interrupt ID, and for an SGI, the source processor ID, that the processor uses to select the correct interrupt handler. When it recognizes this read, the GIC changes the state of the interrupt as follows:

*3 Interrupt Handling and Prioritization 3.2 General handling of interrupts*

• •

if the pending state of the interrupt persists when the interrupt becomes active, or if the interrupt is generated again, from pending to active and pending.

otherwise, from pending to active



When the processor has completed handling the interrupt, it must signal this completion to the GIC. As described in *Priority drop and interrupt deactivation*, this:

- always requires a valid write to an *end of interrupt register* (EOIR)

- might also require a subsequent write to the deactivate interrupt register, GICC_DIR.

  For each CPU interface, the GIC architecture requires the order of the valid writes to an EOIR to be the reverse of the order of the reads from the GICC_IAR or GICC_AIAR, so that each valid EOIR write refers to the most recent interrupt acknowledge.

  If, after the EOIR write, there is no pending interrupt of *Sufficient priority*, the CPU interface deasserts the interrupt exception request to the processor.

  A CPU interface never signals to the connected processor any interrupt that is active and pending. It only signals interrupts that are pending and have sufficient priority:

- For PPIs and SGIs, the active status of particular interrupt ID is banked between CPU interfaces. This means that if a particular interrupt ID is active or active and pending on a CPU interface, then no interrupt with that same ID is signaled on that CPU interface.
- For SPIs, the active status of an interrupt is common to all CPU interfaces. This means that if an interrupt is active or active and pending on one CPU interface then it is not signaled on any CPU interface.

With a GIC implementation that includes the Virtualization Extensions, a hypervisor uses List registers to maintain the list of highest priority virtual interrupts. The total number of interrupts that are either pending, active, or active and pending, can exceed the number of List registers available. If this happens, the hypervisor can save one or more active interrupt entries to memory, and later restore them to the List registers, based on their priority. Therefore:

After the virtual machine completes processing the corresponding virtual interrupt, it writes to the
GICV_EOIR or GICV_AEOIR to deactivate the interrupt. This deactivates both the virtual interrupt and the corresponding physical interrupt, provided that both of the following conditions are true:
• the GICV_CTLR.EOImode bit is set to 0
• the GICH_LRn.HW bit is set to 1.

