#[allow(unused_must_use)];
#[allow(dead_code)];

use std::fmt;

pub trait Mem {
    fn loadb(&self, addr: u16) -> u8;
    fn storeb(&mut self, addr: u16, val: u8) -> bool ;
}

pub trait MemUtil {
    fn loadw(&self, addr: u16) -> u16;
    fn storew(&mut self, addr: u16, val: u16) -> bool;
    fn load(&self, addr: u16, byteflag: bool) -> u16;
    fn store(&mut self, addr: u16, val: u16, byteflag: bool) -> bool ;
}

impl<M: Mem> MemUtil for M {
    fn loadw(&self, addr: u16) -> u16 {
        self.loadb(addr) as u16 | (self.loadb(addr +1) as u16 << 8)
    }

    fn storew(&mut self, addr: u16, val: u16) -> bool {
        self.storeb(addr, (val & 0xff) as u8) && self.storeb(addr + 1, (val >> 8) as u8)
    }

    fn load(&self, addr: u16, byteflag: bool) -> u16 {
        if byteflag {
            self.loadb(addr) as u16
        } else {
            self.loadw(addr) as u16
        }
    }

    fn store(&mut self, addr: u16, val: u16, byteflag: bool) -> bool {
        if byteflag {
            self.storeb(addr, val as u8)
        } else {
            self.storew(addr, val)
        }
    }
}

pub struct Ram {
    arr: [u8, ..0x10000],
    depstatus: bool,
    deparr: [bool, ..0x100], //true = writeable, false = executable
}

impl Ram {
    pub fn new() -> Ram {
        Ram { arr: [0, ..0x10000], depstatus : false, deparr : [false,..0x100] }
    }

    pub fn loadimage(&mut self, image: &[u8]) {
        for (ix, &byte) in image.iter().enumerate() {
            self.storeb(0x4400 + ix as u16, byte);
        }
    }
}

impl Mem for Ram {
    fn loadb(&self, addr: u16) -> u8 {
        self.arr[addr]
    }
    fn storeb(&mut self, addr: u16, val: u8) -> bool {
        if self.depstatus && !self.deparr[addr >> 8] {
            false
        } else {
            self.arr[addr] = val;
            true
        }
    }
}

pub struct Regs {
    arr: [u16, ..16]
}

impl Regs {
    pub fn load(&self, addr: u8) -> u16 {
        self.arr[addr]
    }
    pub fn store(&mut self, addr: u8, val: u16) {
        self.arr[addr] = val
    }
    pub fn new() -> Regs {
        Regs { arr: [0, ..16] }
    }
}

impl fmt::Show for Regs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f.buf, "|-------------Registers-------------|\n");
        write!(f.buf, "|PC  {:04x} SP  {:04x} SR  {:04x} CG  {:04x}|\n",
            self.arr[0], self.arr[1], self.arr[2], self.arr[3]);
        write!(f.buf, "|R04 {:04x} R05 {:04x} R06 {:04x} R07 {:04x}|\n",
            self.arr[4], self.arr[5], self.arr[6], self.arr[7]);
        write!(f.buf, "|R08 {:04x} R09 {:04x} R10 {:04x} R11 {:04x}|\n",
            self.arr[8], self.arr[9], self.arr[10], self.arr[11]);
        write!(f.buf, "|R12 {:04x} R13 {:04x} R14 {:04x} R15 {:04x}|\n",
            self.arr[12], self.arr[13], self.arr[14], self.arr[15]);
        write!(f.buf, "|-----------------------------------|")
    }
}

impl fmt::Show for Ram {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f.buf, "|---------------------RAM----------------------|\n");
        let mut wasvalid = false;
        for i in range(0, self.arr.len()/16) {
            let v = self.arr.slice(16*i, 16*i + 16);
            match v {
                [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0] => {
                    if wasvalid {
                        write!(f.buf, "|                     ***                      |\n");
                        wasvalid = false
                    }
                },
                _ => {
                    write!(f.buf,
                    "|{:04x} : {:02x}{:02x} {:02x}{:02x} {:02x}{:02x} {:02x}{:02x} \
                    {:02x}{:02x} {:02x}{:02x} {:02x}{:02x} {:02x}{:02x}|\n",
                    i*16, v[0], v[1], v[2],v[3],v[4],v[5],v[6],v[7],v[8]
                    ,v[9],v[10],v[11],v[12],v[13],v[14],v[15]);
                    wasvalid = true;
                }
            }
        }
        write!(f.buf, "|----------------------------------------------|")
    }
}
