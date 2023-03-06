use core::fmt;

use crate::mm::{GenericPTE, HostPhysAddr, Level4PageTable, MemFlags, PAGE_SIZE};

bitflags::bitflags! {
    /// Memory attribute fields in the VMSAv8-64 translation table format descriptors.
    pub struct DescriptorAttr: u64 {
        // Attribute fields in stage 2 VMSAv8-64 Block and Page descriptors:

        /// Whether the descriptor is valid.
        const VALID =       1 << 0;
        /// The descriptor gives the address of the next level of translation table or 4KB page.
        /// (not a 2M, 1G block)
        const NON_BLOCK =   1 << 1;
        /// Memory attributes index field.
        const ATTR_INDX =   0b1111 << 2;
        /// Access permission: accessable at EL0/1, Read / Write.
        const S2AP_R      =   1 << 6;
        /// Access permission: accessable at EL0/1, Write.
        const S2AP_W      =   1 << 7;
        /// Shareability: Inner Shareable (otherwise Outer Shareable).
        const INNER     =   1 << 8;
        /// Shareability: Inner or Outer Shareable (otherwise Non-shareable).
        const SHAREABLE =   1 << 9;
        /// The Access flag.
        const AF =          1 << 10;
        /// Indicates that 16 adjacent translation table entries point to contiguous memory regions.
        const CONTIGUOUS =  1 <<  52;
        /// The execute-never field.
        const XN_0 =        1 << 53;
        const XN_1 =        1 << 54;

        // Next-level attributes in stage 2 VMSAv8-64 Table descriptors:(TODO)

        /// PXN limit for subsequent levels of lookup.
        const PXN_TABLE =           1 << 59;
        /// XN limit for subsequent levels of lookup.
        const XN_TABLE =            1 << 60;
        /// Access permissions limit for subsequent levels of lookup: access at EL0 not permitted.
        const AP_NO_EL0_TABLE =     1 << 61;
        /// Access permissions limit for subsequent levels of lookup: write access not permitted.
        const AP_NO_WRITE_TABLE =   1 << 62;
        /// For memory accesses from Secure state, specifies the Security state for subsequent
        /// levels of lookup.
        const NS_TABLE =            1 << 63;
    }
}

#[repr(u64)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum MemType {
    Device = 1,
    Normal = 15,
}

impl DescriptorAttr {
    const ATTR_INDEX_MASK: u64 = 0b1111_00;

    const fn from_mem_type(mem_type: MemType) -> Self {
        let mut bits = (mem_type as u64) << 2;
        if matches!(mem_type, MemType::Normal) {
            bits |= Self::INNER.bits() | Self::SHAREABLE.bits();
        }
        Self::from_bits_truncate(bits)
    }

    fn mem_type(&self) -> MemType {
        let idx = (self.bits() & Self::ATTR_INDEX_MASK) >> 2;
        match idx {
            1 => MemType::Device,
            15 => MemType::Normal,
            _ => panic!("Invalid memory attribute index"),
        }
    }
}

impl From<DescriptorAttr> for MemFlags {
    fn from(attr: DescriptorAttr) -> Self {
        let mut flags = Self::empty();
        if attr.contains(DescriptorAttr::VALID) && attr.contains(DescriptorAttr::S2AP_R) {
            flags |= Self::READ;
        }
        if attr.contains(DescriptorAttr::S2AP_W) {
            flags |= Self::WRITE;
        }
        // now we only consider XN_1.
        if !attr.contains(DescriptorAttr::XN_1) {
            flags |= Self::EXECUTE;
        }
        if attr.mem_type() == MemType::Device {
            flags |= Self::DEVICE;
        }
        flags
    }
}

impl From<MemFlags> for DescriptorAttr {
    fn from(flags: MemFlags) -> Self {
        let mut attr = if flags.contains(MemFlags::DEVICE) {
            Self::from_mem_type(MemType::Device)
        } else {
            Self::from_mem_type(MemType::Normal)
        };
        if flags.contains(MemFlags::READ) {
            attr |= Self::VALID;
            attr |= Self::S2AP_R;
        }
        if flags.contains(MemFlags::WRITE) {
            attr |= Self::S2AP_W;
        }
        if !flags.contains(MemFlags::EXECUTE) {
            attr |= Self::XN_1;
        }
        attr
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    const PHYS_ADDR_MASK: usize = 0xffff_ffff_ffff & !(PAGE_SIZE - 1);

    pub const fn empty() -> Self {
        Self(0)
    }
}

impl GenericPTE for PageTableEntry {
    fn new_page(paddr: HostPhysAddr, flags: MemFlags, is_huge: bool) -> Self {
        let mut attr = DescriptorAttr::from(flags) | DescriptorAttr::AF;
        if !is_huge {
            attr |= DescriptorAttr::NON_BLOCK;
        }
        if paddr == 0x404bb000 {
            info!("Real flags {:?}", attr);
        }
        Self(attr.bits() | (paddr & Self::PHYS_ADDR_MASK) as u64)
    }
    fn new_table(paddr: HostPhysAddr) -> Self {
        let attr = DescriptorAttr::NON_BLOCK
            | DescriptorAttr::VALID
            | DescriptorAttr::ATTR_INDX
            | DescriptorAttr::S2AP_R
            | DescriptorAttr::S2AP_W
            | DescriptorAttr::AF
            | DescriptorAttr::SHAREABLE
            | DescriptorAttr::INNER;
        Self(attr.bits() | (paddr & Self::PHYS_ADDR_MASK) as u64)
    }
    fn paddr(&self) -> HostPhysAddr {
        self.0 as usize & Self::PHYS_ADDR_MASK
    }
    fn flags(&self) -> MemFlags {
        DescriptorAttr::from_bits_truncate(self.0).into()
    }
    fn is_unused(&self) -> bool {
        self.0 == 0
    }
    fn is_present(&self) -> bool {
        DescriptorAttr::from_bits_truncate(self.0).contains(DescriptorAttr::VALID)
    }
    fn is_huge(&self) -> bool {
        !DescriptorAttr::from_bits_truncate(self.0).contains(DescriptorAttr::NON_BLOCK)
    }
    fn clear(&mut self) {
        self.0 = 0
    }
}

impl fmt::Debug for PageTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("PageTableEntry");
        f.field("raw", &self.0)
            .field("paddr", &self.paddr())
            .field("attr", &DescriptorAttr::from_bits_truncate(self.0))
            .field("flags", &self.flags())
            .finish()
    }
}

pub type ExtendedPageTable<H> = Level4PageTable<H, PageTableEntry>;
