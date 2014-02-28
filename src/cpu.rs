use mem::{MemUtil, Ram, Regs};
use std::fmt;
use ncurses;

mod mem;

static CARRYF : u16 = 1;
static ZEROF : u16 = 1 << 1;
static NEGF : u16 = 1 << 2;
static OVERF : u16 = 1 << 8;

// Memory manipulation functions 

pub struct Cpu {
    regs: Regs,
    ram: Ram,
    inst: Instruction,
    status: Status,
    buf: ~str
}

pub struct Instruction {
    //TODO - introduce option types
    memloc: u16,
    code: u16,
    optype: OpType,
    opcode: u8,
    offset: u16,
    bw: bool,
    srcreg: u8,
    destreg: u8,
    srcmode: AddressingMode,
    destmode: AddressingMode
}

enum OpType {
    NoArg,
    OneArg,
    TwoArg,
    Interrupt
}

pub enum Status {
    GetInput(~[u8]),
    Off,
    Success,
    Normal
}

enum AddressingMode {
    Direct,
    Indexed(u16),
    Indirect,
    IndirectInc,
    Absolute(u16),
    Const(u16)
}

impl fmt::Show for AddressingMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self{
            Direct => ~"Direct",
            Indirect => ~"Indirect",
            IndirectInc => ~"IndirectInc",
            Indexed(offset) => format!("Indexed(0x{:x})",offset),
            Absolute(address) => format!("Absolute(0x{:x})",address),
            Const(n) => format!("Const(0x{:x})",n)
        };
        write!(f.buf,"{}", s)
    }
}

fn get_optype(code: u16, pc: u16) -> OpType {
    match (code >> 13, pc) {
        (_,0x10) => Interrupt,
        (0,_) => OneArg,
        (1,_) => NoArg,
        (_,_) => TwoArg
    }
}

//splitters

fn parse_inst(code: u16, pc: u16) -> Instruction {
    let optype = get_optype(code, pc);
    match optype {
        NoArg => noarg_split(code),
        OneArg => onearg_split(code),
        TwoArg => twoarg_split(code),
        Interrupt => { 
            let mut i = twoarg_split(0x4130);
            i.optype = Interrupt;
            i
        }
    }
}

fn twoarg_split(code: u16) -> Instruction {
    let mut inst = Instruction::new();
    inst.code = code;
    inst.optype = TwoArg;
    inst.srcreg = ((code & 0xf00) >> 8) as u8;
    inst.destreg = (code & 0xf) as u8;
    inst.bw = ((code & 0x40) >> 6) != 0;
    inst.opcode = ((code & 0xf000) >> 12) as u8;
    inst
}

fn onearg_split(code: u16) -> Instruction {
    let mut inst = Instruction::new();
    inst.code = code;
    inst.optype = OneArg;
    inst.destreg = (code & 0xf) as u8;
    inst.bw = ((code & 0x40) >> 6) != 0;
    inst.opcode = ((code & 0x380) >> 7) as u8;
    inst
}

fn noarg_split(code: u16) -> Instruction {
    let mut inst = Instruction::new();
    inst.code = code;
    inst.optype = NoArg;
    inst.offset = 2*sxt(code & 0x3ff);
    inst.opcode = ((code & 0x1c00) >> 10) as u8;
    inst
}

impl Cpu {

    fn get_addressing_modes(&mut self) {
        match self.inst.optype {
            TwoArg | Interrupt => {
                self.inst.srcmode = self.modes_(self.inst.srcreg,((self.inst.code & 0x30) >> 4) as u8);
                self.inst.destmode = self.modes_(self.inst.destreg,((self.inst.code & 0x80) >> 7) as u8);
            },
            OneArg => {
                self.inst.destmode = self.modes_(self.inst.destreg,((self.inst.code & 0x30) >> 4) as u8);
            }
            NoArg => ()
        }
    }

    fn modes_(&mut self, reg: u8, modecode: u8) -> AddressingMode {
        match (reg, modecode) {
            (0,0b00) => Direct,
            (0,0b01) => Indexed(self.next_inst()),
            (0,0b10) => Indirect,
            (0,0b11) => Const(self.next_inst()), 
            (2,0b00) => Direct,
            (2,0b01) => Absolute(self.next_inst()),
            (2,0b10) => Const(4),
            (2,0b11) => Const(8),
            (3,0b00) => Const(0),
            (3,0b01) => Const(1),
            (3,0b10) => Const(2),
            (3,0b11) => Const(-1),
            (0..15,0b00) => Direct,
            (0..15,0b01) => Indexed(self.next_inst()),
            (0..15,0b10) => Indirect,
            (0..15,0b11) => IndirectInc,
            (_,_) => {
                ncurses::endwin();
                println!("{}", self.inst);
                fail!(format!("Invalid register/mode combo: Reg {} Mode {}", reg, modecode))
            }
        }
    }


    // memory/register interface
    
    //turn indirects into values
    fn resolve(&mut self, regadr: u8, mode: AddressingMode) -> u16 {
        let regval = self.regs.load(regadr);
        let mut val = match mode {
            Direct => regval,
            Indirect => self.ram.load(regval, self.inst.bw),
            IndirectInc => {
                self.regs.store(regadr, regval + 2);
                self.ram.load(regval, self.inst.bw)
            }
            Indexed(offset) => {
                self.ram.load(regval + offset, self.inst.bw)
            }
            Absolute(address) => self.ram.load(address, self.inst.bw),
            Const(n) => n
        };
        if self.inst.bw { val &= 0xff };
        val
    }

    fn _store(&mut self, regadr: u8, mode: AddressingMode, val: u16) {
        let regval = self.regs.load(regadr);
        match mode {
            Direct => self.regs.store(regadr, val),
            Indirect => self.ram.store(regval, val, self.inst.bw),
            IndirectInc => {
                self.regs.store(regadr, regval + 1);
                self.ram.store(regval, val, self.inst.bw)
            }
            Indexed(offset) => {
                self.ram.store(regval + offset, val, self.inst.bw )
            },
            Absolute(address) => {
                self.ram.store(address, val, self.inst.bw)
            },
            _ => fail!("Invalid addressing mode")
        }
    }

    //wrapper
    fn store(&mut self, val: u16) {
        self._store(self.inst.destreg, self.inst.destmode, val)
    }

    //execution stage
    fn exec(&mut self) {
        match (self.inst.optype,self.inst.opcode) {
            (NoArg,0b000) => self.noarg_dispatch(JNE),
            (NoArg,0b001) => self.noarg_dispatch(JEQ),
            (NoArg,0b010) => self.noarg_dispatch(JNC),
            (NoArg,0b011) => self.noarg_dispatch(JC),
            (NoArg,0b100) => self.noarg_dispatch(JN),
            (NoArg,0b101) => self.noarg_dispatch(JGE),
            (NoArg,0b110) => self.noarg_dispatch(JL),
            (NoArg,0b111) => self.noarg_dispatch(JMP),
            (OneArg,0b000) => self.onearg_dispatch(RRC),
            (OneArg,0b001) => self.onearg_dispatch(SWPB),
            (OneArg,0b010) => self.onearg_dispatch(RRA),
            (OneArg,0b011) => self.onearg_dispatch(SXT),
            (OneArg,0b100) => self.onearg_dispatch(PUSH),
            (OneArg,0b101) => self.onearg_dispatch(CALL),
            (OneArg,0b110) => self.onearg_dispatch(RETI),
            (TwoArg,0b0100) => self.twoarg_dispatch(MOV),
            (TwoArg,0b0101) => self.twoarg_dispatch(ADD),
            (TwoArg,0b0110) => self.twoarg_dispatch(ADDC),
            (TwoArg,0b0111) => self.twoarg_dispatch(SUBC),
            (TwoArg,0b1000) => self.twoarg_dispatch(SUB),
            (TwoArg,0b1001) => self.twoarg_dispatch(CMP),
            (TwoArg,0b1010) => self.twoarg_dispatch(DADD),
            (TwoArg,0b1011) => self.twoarg_dispatch(BIT),
            (TwoArg,0b1100) => self.twoarg_dispatch(BIC),
            (TwoArg,0b1101) => self.twoarg_dispatch(BIS),
            (TwoArg,0b1110) => self.twoarg_dispatch(XOR),
            (TwoArg,0b1111) => self.twoarg_dispatch(AND),
            (Interrupt,_) => self.handle_interrupt(),
            _ => fail!("Illegal opcode")
        }
    }

    fn noarg_dispatch(&mut self, f: fn(&Cpu) -> bool) {
        if f(self) { self.regs.arr[0] = self.regs.arr[0] + self.inst.offset }
    }

    fn onearg_dispatch(&mut self, f: fn(&mut Cpu, val: u16)) {
        let val = self.resolve(self.inst.destreg, self.inst.destmode);
        f(self, val)
    }

    fn twoarg_dispatch(&mut self, f: fn(&mut Cpu, val: u16, inc:u16)) {
        let inc = self.resolve(self.inst.srcreg, self.inst.srcmode);
        let val = self.resolve(self.inst.destreg, self.inst.destmode);
        f(self, val, inc)
    }

    fn handle_interrupt(&mut self) {
        match self.regs.arr[2] {            //sr register
            0x8000 => {
                self.buf.push_char(self.ram.arr[self.regs.arr[1]+8] as char);
                self.twoarg_dispatch(MOV)
            }
            0x8200 => { self.status = GetInput(~[]) },                     //getsn 
            0xff00 => { self.status = Success }
            0xfd00 => { self.twoarg_dispatch(MOV) }
            _ => ()
        }
    }
}


//Instructions

//No arg

fn JNE(cpu : &Cpu) -> bool { if !cpu.getflag(ZEROF) { true } else { false } }
fn JEQ(cpu : &Cpu) -> bool { if cpu.getflag(ZEROF) { true } else { false } }
fn JNC(cpu : &Cpu) -> bool { if !cpu.getflag(CARRYF) { true } else { false } }
fn JC(cpu : &Cpu) -> bool { if !cpu.getflag(CARRYF) { true } else { false } }
fn JN(cpu : &Cpu) -> bool { if cpu.getflag(NEGF) { true } else { false } }
fn JGE(cpu : &Cpu) -> bool  { if cpu.getflag(NEGF) == cpu.getflag(OVERF) { true } else {false} }
fn JL(cpu : &Cpu) -> bool { if !(cpu.getflag(NEGF) == cpu.getflag(OVERF)) { true } else { false } }
fn JMP(cpu : &Cpu) -> bool { true }

// One arg

//XXX think this is wrong
fn RRC(cpu: &mut Cpu, mut val: u16) {
    let C = cpu.getflag(CARRYF);
    val >>= 1;
    if C { val |= 0x8000 }
    cpu.set_and_store(val)
}

fn SWPB(cpu: &mut Cpu, val: u16) {
    let topbyte = val >> 8;
    let botbyte = val << 8;
    cpu.store(topbyte | botbyte)
}

//TODO: implement
fn RRA(cpu:&mut Cpu, val: u16) { fail!("Not implemented") }

fn sxt(mut val: u16) -> u16 {
    if (val & 0x0080) != 0 { val |= 0xff00 }
    val
}

fn SXT(cpu:&mut Cpu, mut val: u16) {
    cpu.store(sxt(val))
}

fn PUSH(cpu:&mut Cpu, val: u16) {
    cpu.regs.arr[1] -= 2;
    cpu._store(1, Indirect, val);        //push 
}

//XXX: broken
fn CALL(cpu:&mut Cpu,val: u16) {
    //val is location of branch
    cpu.inst.destreg = 0;
    cpu.inst.destmode = Direct;
    PUSH(cpu,cpu.regs.arr[0]); // push pc to stack 
    cpu.regs.arr[0] = val
}

fn RETI(cpu:&mut Cpu, val: u16) {
    fail!("Not implemented")
}

// Two arg

fn ADDC(cpu:&mut Cpu, val: u16, inc: u16) {
    let C = cpu.getflag(CARRYF);
    if C { cpu.set_and_store(val + inc + 1) } else { cpu.set_and_store(val + inc) }
}

fn SUBC(cpu:&mut Cpu, val: u16, inc: u16) {
    let C = cpu.getflag(CARRYF);
    if C { cpu.set_and_store(val - inc + 1) } else { cpu.set_and_store(val - inc) }
}

fn MOV(cpu: &mut Cpu, val: u16, inc: u16) { cpu.store(inc) }
fn ADD(cpu: &mut Cpu, val: u16, inc: u16) { cpu.set_and_store(val + inc) }
fn SUB(cpu: &mut Cpu, val: u16, inc: u16) { cpu.set_and_store(val - inc) }
fn CMP(cpu: &mut Cpu, val: u16, inc: u16) { cpu.setzn(val - inc); }
fn BIT(cpu: &mut Cpu, val: u16, inc: u16) { cpu.setzn(inc & val); } 
fn BIC(cpu: &mut Cpu, val: u16, inc: u16) { cpu.store(val & !inc) }
fn BIS(cpu: &mut Cpu, val: u16, inc: u16) { cpu.store(val | inc) }
fn XOR(cpu: &mut Cpu, val: u16, inc: u16) { cpu.set_and_store(val ^ inc) }
fn AND(cpu: &mut Cpu, val: u16, inc: u16) { cpu.set_and_store(val & inc) }
fn DADD(cpu:&mut Cpu, val: u16, inc: u16) { fail!("Not implemented") }

impl Cpu {

    // utility functions
    fn getflag(&self, flag: u16) -> bool {
        if (self.regs.arr[2] & flag) == 0 {
            false
        } else {
            true
        }
    }

    fn set_flag(&mut self, flag: u16, on: bool ) {
        if on {
            self.regs.arr[2] = self.regs.arr[2] | flag
        } else {
            self.regs.arr[2] = self.regs.arr[2] & !flag
        }
    }

    pub fn swap(i: u16) -> u16 {
        let ret = i >> 8;
        ret | i << 8
    }

    fn setzn(&mut self, val: u16) {
        self.set_flag(ZEROF, val == 0);
        self.set_flag(NEGF, val & 0x8000 != 0);
    }

    fn set_and_store(&mut self, val: u16) {
        self.setzn(val);
        self.store(val);
    }

    // load instruction from ram and increment pc
    fn next_inst(&mut self) -> u16 {
        let inst = self.ram.loadw(self.regs.arr[0]);
        self.regs.arr[0] += 2;
        if !self.regs.arr[0] % 2 == 0 {
            ncurses::endwin();
            println!("{}", self.inst);
            fail!(format!("Invalid address {}", self.regs.arr[0]))
        }
        inst
    }

    // load and execute one instruction
    pub fn step(&mut self) { 
        let mut b = ~[];
        match self.status {
            Normal => {
                self.exec();
                self.prepare_next();
                if self.regs.arr[2] & 0x80 != 0 { self.status = Off } // CPU OFF
            },
            Off | Success => (),
            GetInput(ref bytes) => b = bytes.clone()
        }
        if b != ~[] {
            self.getsn(b);
            //prepare next instruction
            self.status = Normal;
            self.inst =  parse_inst(0x4130,0);
            self.get_addressing_modes();
        }
    }

    fn getsn(&mut self, bytes: ~[u8]) {
        let sp = self.regs.arr[1];
        let putloc = self.ram.loadw(sp + 8);
        let mut getn = self.ram.loadw(sp + 10);
        if (bytes.len() as u16) < getn { getn = bytes.len() as u16 }
        for i in range(0, getn) {
            self.ram.arr[putloc + (i as u16)] =  bytes[i];
        }
    }

    fn prepare_next(&mut self) {
        let pc = self.regs.arr[0];
        let code = self.next_inst();
        self.inst = parse_inst(code, pc);
        self.get_addressing_modes();
        self.inst.memloc = pc;
    }


    pub fn new() -> Cpu { 
        Cpu {
            regs: Regs::new(),
            ram: Ram::new(),
            inst: Instruction::new(),
            status: Normal,
            buf: ~""
        }
    }


    pub fn init(image: &[u8]) -> Cpu {
        let mut cpu = Cpu::new();
        cpu.ram.loadimage(image);
        cpu.regs.arr[0] = 0x4400;
        cpu.prepare_next();
        cpu
    }
}

impl fmt::Show for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f.buf,
"******************* CPU *******************
{}

{}

{}

{}
++++++++++++++++++++++++++++++++++++++++++++", self.ram, self.regs, self.inst, self.buf)
    }
}

impl Instruction {
    fn new() -> Instruction {
        Instruction {
            memloc: 0,
            code: 0,
            optype: NoArg,
            opcode: 0,
            offset: 0,
            bw: false,
            destmode: Direct,
            srcmode: Direct,
            destreg: 0,
            srcreg: 0,
        }
    }

    fn namer(&self) -> ~str {
        match (self.optype, self.opcode) {
            (NoArg,0b000) => ~"JNE",
            (NoArg,0b001) => ~"JEQ",
            (NoArg,0b010) => ~"JNC",
            (NoArg,0b011) => ~"JC",
            (NoArg,0b100) => ~"JN",
            (NoArg,0b101) => ~"JGE",
            (NoArg,0b110) => ~"JL",
            (NoArg,0b111) => ~"JMP",
            (OneArg,0b000) => ~"RRC",
            (OneArg,0b001) => ~"SWPB",
            (OneArg,0b010) => ~"RRA",
            (OneArg,0b011) => ~"SXT",
            (OneArg,0b100) => ~"PUSH",
            (OneArg,0b101) => ~"CALL",
            (OneArg,0b110) => ~"RETI",
            (TwoArg,0b0100) => ~"MOV",
            (TwoArg,0b0101) => ~"ADD",
            (TwoArg,0b0110) => ~"ADDC",
            (TwoArg,0b0111) => ~"SUBC",
            (TwoArg,0b1000) => ~"SUB",
            (TwoArg,0b1001) => ~"CMP",
            (TwoArg,0b1010) => ~"DADD",
            (TwoArg,0b1011) => ~"BIT",
            (TwoArg,0b1100) => ~"BIC",
            (TwoArg,0b1101) => ~"BIS",
            (TwoArg,0b1110) => ~"XOR",
            (TwoArg,0b1111) => ~"AND",
            (Interrupt,_) => ~"INT",
            (_,_) => unreachable!()
        }
    }

    pub fn to_string(&self) -> ~str {
        let op = self.namer();
        let byte = if self.bw { ~".B" } else { ~"" };
        let (a1, a2) = match self.optype {
            NoArg => (format!("\\#0x{:04x}", self.offset + 2), ~""),
            OneArg => (optype_formatter(self.destmode, self.destreg), ~""),
            TwoArg => (optype_formatter(self.srcmode, self.srcreg),
                       optype_formatter(self.destmode, self.destreg)),
            Interrupt => (~"",~"")

        };
        format!("{:s}{:s} {:s} {:s}", op, byte, a1, a2)
    }
}

fn optype_formatter(mode: AddressingMode, reg: u8) -> ~str {
    match mode {
        Direct => format!("r{:u}", reg),
        Indirect => format!("@r{:u}", reg),
        IndirectInc => format!("@r{:u}+", reg),
        Absolute(address) => format!("&0x{:x}", address),
        Indexed(offset) => format!("(0x{:x})r{:u}", offset, reg),
        Const(n) => format!("{:x}", n)
    }
}

impl fmt::Show for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f.buf, 
"|-------- Instruction: 0x{:04x}//{:016t}-----------|
| OpType:{:06?} | Opcode:{:04t} | B/W:{:05b} | Offset: {:04x}  | 
| DestReg:  {:02u}  | DestMode:  {:11?} | MemLoc: {:04x}
| SourceReg:{:02u}  | SourceMode:{:11?} |
|--------------          {:20s}-------------|",
               self.code,self.code,
               self.optype, self.opcode, self.bw, self.offset,
               self.destreg, self.destmode, self.memloc,
               self.srcreg, self.srcmode, self.to_string())
    }
}
