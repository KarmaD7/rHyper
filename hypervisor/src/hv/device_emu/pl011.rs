use rvm::RvmResult;
use spin::Mutex;

use crate::device::{console_getchar, console_putchar};

use super::MMIODevice;

const PL011_DR: usize = 0x00;
const PL011_FR: usize = 0x18;
const PL011_CR: usize = 0x30;
const PL011_IFLS: usize = 0x34;
const PL011_IMSC: usize = 0x38;
const PL011_RIS: usize = 0x3c;
const PL011_MIS: usize = 0x40;
const PL011_ICR: usize = 0x44;

const UART_FIFO_CAPACITY: usize = 16;

bitflags::bitflags! {
    /// Line status flags
    struct LineStsFlags: u8 {
        // 0 to 3 unknown
        const INPUT_EMPTY = 1 << 4;
        const OUTPUT_FULL = 1 << 5;
        // 6 and 7 unknown
    }
}

struct Fifo<const CAP: usize> {
    buf: [u8; CAP],
    head: usize,
    num: usize,
}

impl<const CAP: usize> Fifo<CAP> {
    const fn new() -> Self {
        Self {
            buf: [0; CAP],
            head: 0,
            num: 0,
        }
    }

    fn is_empty(&self) -> bool {
        self.num == 0
    }

    fn is_full(&self) -> bool {
        self.num == CAP
    }

    fn push(&mut self, value: u8) {
        assert!(self.num < CAP);
        self.buf[(self.head + self.num) % CAP] = value;
        self.num += 1;
    }

    fn pop(&mut self) -> u8 {
        assert!(self.num > 0);
        let ret = self.buf[self.head];
        self.head += 1;
        self.head %= CAP;
        self.num -= 1;
        ret
    }
}

pub struct Pl011 {
    base_vaddr: usize,
    fifo: Mutex<Fifo<UART_FIFO_CAPACITY>>,
}

impl Pl011 {
    pub const fn new(base_vaddr: usize) -> Self {
        Self {
            base_vaddr,
            fifo: Mutex::new(Fifo::new()),
        }
    }
}

impl MMIODevice for Pl011 {
    fn mem_range(&self) -> core::ops::Range<usize> {
        self.base_vaddr..self.base_vaddr + 0x1000
    }

    fn read(&self, addr: usize, access_size: u8) -> RvmResult<u32> {
        debug!("pl011 read mock, addr: {:#x}", addr);
        let ret = match addr - self.base_vaddr {
            PL011_DR => {
                let mut fifo = self.fifo.lock();
                if fifo.is_empty() {
                    0
                } else {
                    fifo.pop()
                }
            }
            PL011_FR => {
                let mut fifo = self.fifo.lock();
                let mut fr = LineStsFlags::empty();

                if !fifo.is_full() {
                    if let Some(b) = console_getchar() {
                        fifo.push(b)
                    }
                }

                if fifo.is_empty() {
                    fr |= LineStsFlags::INPUT_EMPTY;
                }

                fr.bits()
            }
            _ => unreachable!(),
        };
        debug!("ret {:x}", ret);
        Ok(ret as u32)
    }

    fn write(&self, addr: usize, val: u32, access_size: u8) -> RvmResult {
        debug!("pl011 write mock, addr: {:#x}", addr);
        match addr - self.base_vaddr {
            PL011_DR => console_putchar(val as u8),
            PL011_FR => {}
            _ => {}
        }
        Ok(())
    }
}
