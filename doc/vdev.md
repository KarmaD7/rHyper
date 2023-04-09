1) 对于Passthrough的外设，hypervisor会将对应的外设地址map到IPA空间，这样VM中的software就可以与外设直接交互。

2) 对于虚拟外设，hypervisor会将对应的stage2 table entries标记为Fault.VM中软件以为它在直接与外设交互，但实际上每次交互都会触发一次stage2 fault，并伴随着hypervisor在异常处理函数中模拟外设进行交互。

3) 通常处理stag1 fault时，我们可以从寄存器FAR_ELx中获取触发异常的va。但是对于hypervisor来说，获取vm中的va并没有太大的意义。因此对于stag2 fault，arm提供了HPFAR_EL2来提供触发异常的IPA，异常相关的信息存放在寄存器ESR_EL2中

To signal virtual interrupts to EL0/1, a hypervisor must set the corresponding routing bit in HCR_EL2. For example, to enable vIRQ signaling, a hypervisor must set HCR_EL2.IMO. This setting routes physical IRQ exceptions to EL2, and enables signaling of the virtual exception to EL1.
Virtual interrupts are controlled per interrupt type. In theory, a VM could be configured to receive physical FIQs and virtual IRQs. In practice, this is unusual. A VM is usually configured only to receive virtual interrupts.

1. The physical peripheral asserts its interrupt signal into the GIC.
2. The GIC generates a physical interrupt exception, either IRQ or FIQ, which gets routed to EL2 by the configuration of HCR_EL2.IMO/FMO. The hypervisor identifies the peripheral and determines that it has been assigned to a VM. It checks which vCPU the interrupt should be forwarded to.
3. The hypervisor configures the GIC to forward the physical interrupt as a virtual interrupt to the vCPU. The GIC will then assert the vIRQ or vFIQ signal, but the processor will ignore this signal while it is executing in EL2.
4. The hypervisor returns control to the vCPU.
5. Now that the processor is in the vCPU (EL0 or EL1), the virtual interrupt from the GIC can be
taken. This virtual interrupt is subject to the PSTATE exception masks.

mmio handle dabt:

	u32 iss		= ESR_ISS(ctx->esr);
	u32 isv		= iss >> 24;
	u32 sas		= iss >> 22 & 0x3;
	u32 sse		= iss >> 21 & 0x1;
	u32 srt		= iss >> 16 & 0x1f;
	u32 ea		= iss >> 9 & 0x1;
	u32 cm		= iss >> 8 & 0x1;
	u32 s1ptw	= iss >> 7 & 0x1;
	u32 is_write	= iss >> 6 & 0x1;
	u32 size	= 1 << sas;