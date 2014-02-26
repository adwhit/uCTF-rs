use mem::{MemUtil, Ram, Regs};
use std::fmt;

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
    destmode: AddressingMode,
}

enum OpType {
    NoArg,
    OneArg,
    TwoArg
}

enum AddressingMode {
    Direct,
    Indexed(u16),
    Indirect,
    IndirectInc,
    Absolute(u16),
    Const(u16)
}

fn get_optype(code: u16) -> OpType {
    match code >> 13 {
        0 => OneArg,
        1 => NoArg,
        _ => TwoArg
    }
}

//splitters

fn parse_inst(code: u16) -> Instruction {
    let optype = get_optype(code);
    match optype {
        NoArg => noarg_split(code),
        OneArg => onearg_split(code),
        TwoArg => twoarg_split(code)
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
        self.inst.srcmode = self.modes_(((self.inst.code & 0x30) >> 4) as u8, self.inst.srcreg);
        self.inst.destmode = self.modes_(((self.inst.code & 0x80) >> 7) as u8, self.inst.destreg);
    }

    fn modes_(&mut self, modecode: u8, reg: u8) -> AddressingMode {
        match (reg, modecode) {
            (2,0b00) => Direct,
            (2,0b01) => Absolute(self.next_inst()),
            (2,0b10) => Const(4),
            (2,0b11) => Const(8),
            (3,0b00) => Const(0),
            (3,0b01) => Const(1),
            (3,0b10) => Const(2),
            (3,0b11) => Const(-1),
            (0,0b11) => Const(self.next_inst()),
            (0..15,0b00) => Direct,
            (0..15,0b01) => Indexed(self.next_inst()),
            (0..15,0b10) => Indirect,
            (1..15,0b11) => IndirectInc,
            (_,_) => fail!("Invalid register")
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

    fn store(&mut self, val: u16) {
        self._store(self.inst.destreg, self.inst.destmode, val)
    }

    fn set_and_store(&mut self, val: u16) {
        self.setflags(val);
        self.store(val);
    }
    
    fn exec(&mut self) {
        match self.inst.optype {
            NoArg => { 
                let f = match self.inst.opcode {
                    0b000 => JNE,
                    0b001 => JEQ,
                    0b010 => JNC,
                    0b011 => JC,
                    0b100 => JN,
                    0b101 => JGE,
                    0b110 => JL,
                    0b111 => JMP,
                    _ => fail!("Illegal opcode")
                };
                self.noarg_dispatch(f)
            }
            OneArg => {
                let f = match self.inst.opcode {
                    0b000 => RRC,
                    0b001 => SWPB,
                    0b010 => RRA,
                    0b011 => SXT,
                    0b100 => PUSH,
                    0b101 => CALL,
                    0b110 => RETI,
                    _ => fail!("Illegal opcode")
                };
                self.onearg_dispatch(f)
            }
            TwoArg => {
                let f = match self.inst.opcode {
                    0b0100 => MOV,
                    0b0101 => ADD,
                    0b0110 => ADDC,
                    0b0111 => SUBC,
                    0b1000 => SUB,
                    0b1001 => CMP,
                    0b1010 => DADD,
                    0b1011 => BIT,
                    0b1100 => BIC,
                    0b1101 => BIS,
                    0b1110 => XOR,
                    0b1111 => AND,
                    _ => fail!("Illegal opcode")
                };
                self.twoarg_dispatch(f)
            }
        }
    }

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

    fn setflags(&mut self, val: u16) {
        self.set_flag(ZEROF, val == 0);
        self.set_flag(NEGF, val & 0x8000 != 0);
    }

    // load instruction from ram and increment pc
    fn next_inst(&mut self) -> u16 {
        let inst = self.ram.loadw(self.regs.arr[0]);
        self.regs.arr[0] += 2;
        assert!(self.regs.arr[0] % 2 == 0);
        inst
    }

    // load and execute one instruction
    pub fn step(&mut self) { 
        self.exec();
        self.prepare_next();
    }

    fn prepare_next(&mut self) {
        let code = self.next_inst();
        let pc = self.regs.arr[0];
        self.inst = parse_inst(code);
        self.inst.memloc = pc - 2;
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

    pub fn new() -> Cpu { 
        Cpu {
            regs: Regs::new(),
            ram: Ram::new(),
            inst: Instruction::new(),
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

fn MOV(cpu: &mut Cpu, val: u16, inc: u16) { cpu.buf.push_str(format!("Moving! 0x{:x}\n", inc)); cpu.store(inc) }
fn ADD(cpu: &mut Cpu, val: u16, inc: u16) { cpu.set_and_store(val + inc) }
fn SUB(cpu: &mut Cpu, val: u16, inc: u16) { cpu.set_and_store(val - inc) }
fn CMP(cpu: &mut Cpu, val: u16, inc: u16) { cpu.setflags(val - inc); }
fn BIT(cpu: &mut Cpu, val: u16, inc: u16) { cpu.setflags(inc & val); } 
fn BIC(cpu: &mut Cpu, val: u16, inc: u16) { cpu.store(val & !inc) }
fn BIS(cpu: &mut Cpu, val: u16, inc: u16) { cpu.store(val | inc) }
fn XOR(cpu: &mut Cpu, val: u16, inc: u16) { cpu.set_and_store(val ^ inc) }
fn AND(cpu: &mut Cpu, val: u16, inc: u16) { cpu.set_and_store(val & inc) }
fn DADD(cpu:&mut Cpu, val: u16, inc: u16) { fail!("Not implemented") }

pub fn swap(i: u16) -> u16 {
    let ret = i >> 8;
    ret | i << 8
}

impl fmt::Show for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f.buf,
"******************* CPU *******************
{}

{}

{}
++++++++++++++++++++++++++++++++++++++++++++", self.ram, self.regs, self.inst)
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
        match self.optype {
            NoArg => match self.opcode {
                0b000 => ~"JNE",
                0b001 => ~"JEQ",
                0b010 => ~"JNC",
                0b011 => ~"JC",
                0b100 => ~"JN",
                0b101 => ~"JGE",
                0b110 => ~"JL",
                0b111 => ~"JMP",
                _ => fail!("Illegal opcode")
                },
            OneArg => match self.opcode {
                0b000 => ~"RRC",
                0b001 => ~"SWPB",
                0b010 => ~"RRA",
                0b011 => ~"SXT",
                0b100 => ~"PUSH",
                0b101 => ~"CALL",
                0b110 => ~"RETI",
                _ => fail!("Illegal opcode")
                },
            TwoArg => match self.opcode {
                0b0100 => ~"MOV",
                0b0101 => ~"ADD",
                0b0110 => ~"ADDC",
                0b0111 => ~"SUBC",
                0b1000 => ~"SUB",
                0b1001 => ~"CMP",
                0b1010 => ~"DADD",
                0b1011 => ~"BIT",
                0b1100 => ~"BIC",
                0b1101 => ~"BIS",
                0b1110 => ~"XOR",
                0b1111 => ~"AND",
                _ => fail!("Illegal opcode")
            }
        }
    }

    pub fn to_string(&self) -> ~str {
        let op = self.namer();
        let byte = if self.bw { ~".B" } else { ~"" };
        let (a1, a2) = match self.optype {
            NoArg => (format!("\\#0x{:04x}", self.offset + 2), ~""),
            OneArg => (optype_formatter(self.destmode, self.destreg), ~""),
            TwoArg => (optype_formatter(self.srcmode, self.srcreg),
                       optype_formatter(self.destmode, self.destreg))
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
| DestReg:  {:02u}  | DestMode:  {:11?} |
| SourceReg:{:02u}  | SourceMode:{:11?} |
|--------------          {:20s}-------------|",
               self.code,self.code,
               self.optype, self.opcode, self.bw, self.offset,
               self.destreg, self.destmode,
               self.srcreg, self.srcmode, self.to_string())
    }
}

#[test]
fn parse_tests() {
    let instrs: ~[u16] =         ~[0x4031,0x37ff,0x118b]; //MOV, JGE, SXT
    let optype: ~[OpType]=       ~[TwoArg, NoArg, OneArg];
    let opcodes: ~[u8]=          ~[0b0100, 0b101, 0b011];
    let sourceregs: ~[u8]=       ~[0, 0, 0];
    let Ads: ~[AddressingMode] = ~[Direct, Direct, Direct];
    let bws: ~[bool] =           ~[false, false, false];
    let Ass: ~[AddressingMode] = ~[IndirectInc, Direct, Direct];
    let destregs: ~[u8] =        ~[0b0001, 0, 11];
    for (ix, &code) in instrs.iter().enumerate() {
        let inst = parse_inst(code);
        //println!("{}", inst);
        assert_eq!(inst.opcode, opcodes[ix]);
        assert_eq!(inst.optype as u8, optype[ix] as u8);
        assert_eq!(inst.srcreg, sourceregs[ix]);
        assert_eq!(inst.destmode as u8, Ads[ix] as u8);
        assert_eq!(inst.bw, bws[ix]);
        assert_eq!(inst.srcmode as u8, Ass[ix] as u8);
        assert_eq!(inst.destreg, destregs[ix]);
    }
}

#[test]
fn cpu_test() {
    let mut cpu = Cpu::new();
    let v: ~[u8] = ~[0x31,0x40,0x00,0x44,0x15,0x42,0x5c,0x01,
          0x75,0xf3,0x35,0xd0,0x08,0x5a];
    for (ix, &val) in v.iter().enumerate() {
        cpu.ram.arr[ix] = val
    }
    cpu.prepare_next();
    println!("{}\n", cpu);
    cpu.step();
    println!("{}\n", cpu);

}
