/*
000 RRC(.B) 9-bit rotate right through carry. 
001 SWPB    Swap 8-bit register halves. No byte form.
010 RRA(.B) Badly named, this is an 8-bit arithmetic right shift.
011 SXT Sign extend 8 bits to 16. No byte form.
100 PUSH(.B)    Push operand on stack. 
101 CALL    Fetch operand, push PC,  assign operand value to PC. 
110 RETI    Pop SP, then pop PC. 
111 Not used    

0100    MOV src,dest    dest = src  The status flags are NOT set.
0101    ADD src,dest    dest += src  
0110    ADDC src,dest   dest += src + C  
0111    SUBC src,dest   dest += ~src + C     
1001    SUB src,dest    dest -= src Impltd as dest += ~src + 1.
1001    CMP src,dest    dest - src  Sets status only 
1010    DADD src,dest   dest += src + C, BCD.    
1011    BIT src,dest    dest & src  Sets status only 
1100    BIC src,dest    dest &= ~src  Status flags are NOT set.
1101    BIS src,dest    dest |= src   Status flags are NOT set.
1110    XOR src,dest    dest ^= src  
1111    AND src,dest    dest &=- src

000 JNE/JNZ Jump if Z==0 (if !=)
001 JEQ/Z   Jump if Z==1 (if ==)
010 JNC/JLO Jump if C==0 (if unsigned <)
011 JC/JHS  Jump if C==1 (if unsigned >=)
100 JN  Jump if N==1 Note there is no "JP" if N==0!
101 JGE Jump if N==V (if signed >=)
110 JL  Jump if N!=V (if signed <)
111 JMP Jump unconditionally

*/

//Flags

static CARRYF : u16 = 1;
static ZEROF : u16 = 1 << 1;
static NEGF : u16 = 1 << 2;
static OVERF : u16 = 1 << 8;

// Memory manipulation functions 

trait Mem {
    fn loadb(&self, addr: u16) -> u8;
    fn storeb(&mut self, addr: u16, val: u8);
}

trait MemUtil {
    fn loadw(&self, addr: u16) -> u16;
    fn storew(&mut self, addr: u16, val: u16);
    fn load(&self, addr: u16, byteflag: bool) -> u16;
    fn store(&mut self, addr: u16, val: u16, byteflag: bool);
}

impl<M: Mem> MemUtil for M {
    fn loadw(&self, addr: u16) -> u16 {
        self.loadb(addr) as u16 | (self.loadb(addr +1) as u16 << 8)
    }

    fn storew(&mut self, addr: u16, val: u16) {
        self.storeb(addr, (val & 0xff) as u8);
        self.storeb(addr + 1, (val >> 8) as u8);
    }

    fn load(&self, addr: u16, byteflag: bool) -> u16 {
        if byteflag {
            self.loadb(addr) as u16
        } else {
            self.loadw(addr) as u16
        }
    }

    fn store(&mut self, addr: u16, val: u16, byteflag: bool) {
        if byteflag {
            self.storeb(addr, val as u8)
        } else {
            self.storew(addr, val)
        }
    }
}

struct Cpu {
    regs: Regs,
    ram: Ram,
    inst: Instruction
}

struct Ram {
    arr: [u8, ..0x10000],
}

impl Ram {
    fn new() -> Ram {
        Ram { arr: [0, ..0x10000] }
    }
}

struct Regs {
    arr: [u16, ..15]
}


impl Mem for Ram {
    fn loadb(&self, addr: u16) -> u8 {
        self.arr[addr]
    }
    fn storeb(&mut self, addr: u16, val: u8) {
        self.arr[addr] = val
    }
}

impl Regs {
    fn load(&self, addr: u8) -> u16 {
        self.arr[addr]
    }
    fn store(&mut self, addr: u8, val: u16) {
        self.arr[addr] = val
    }
    fn new() -> Regs {
        Regs { arr: [0, ..15] }
    }
}

struct Instruction {
    code: u16,
    optype: OpType,
    opcode: u8,
    offset: u16,
    bw: bool,
    Ad: AddressingMode,
    As: AddressingMode,
    sourcereg: u8,
    destreg: u8
}

impl Instruction {
    fn new() -> Instruction {
        Instruction {
            code: 0,
            optype: NoArg,
            opcode: 0,
            offset: 0,
            bw: false,
            Ad: Direct,
            As: Direct,
            sourcereg: 0,
            destreg: 0
        }
    }
}

enum OpType {
    NoArg,
    OneArg,
    TwoArg
}

enum AddressingMode {
    Direct,
    Indirect,
    IndirectInc,
    Indexed
}

fn get_optype(code: u16) -> OpType {
    println!("{:016t}, {:u}",code, code >> 12)
    match code >> 12 {
        0 => NoArg,
        1 => OneArg,
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

fn twoarg_split(inst: u16) -> Instruction {
    let destreg = (inst & 0xf) as u8;
    let bw = ((inst & 0x40) >> 6) != 0;
    let As = getAddressingMode(((inst & 0x30) >> 4) as u8);
    let Ad = getAddressingMode(((inst & 0x80) >> 7) as u8);
    let sourcereg = ((inst & 0xf00) >> 8) as u8;
    let opcode = ((inst & 0xf000) >> 12) as u8;
    Instruction { 
        code: inst,
        optype: TwoArg,
        opcode: opcode, 
        destreg: destreg,
        sourcereg: sourcereg,
        As : As,
        Ad : Ad,
        bw : bw,
        offset : 0
    }
}

fn onearg_split(inst: u16) -> Instruction {
    let destreg = (inst & 0xf) as u8;
    let Ad = getAddressingMode(((inst & 0x30) >> 4) as u8);
    let bw = ((inst & 0x40) >> 6) == 0;
    let opcode = ((inst & 0x380) >> 7) as u8;
    Instruction { 
        code: inst,
        optype: OneArg,
        opcode: opcode, 
        destreg: destreg,
        Ad : Ad,
        bw : bw,
        sourcereg : 0,
        offset : 0,
        As : Direct,
    }
}

fn noarg_split(inst: u16) -> Instruction {
    let offset = (inst & 0x1ff);
    let opcode = ((inst & 0x1c00) >> 7) as u8;
    Instruction { 
        code: inst,
        optype: NoArg,
        opcode: opcode,
        offset: offset,
        bw : false,
        Ad : Indirect,
        As: Indirect,
        sourcereg : 0,
        destreg : 0,
    }
}

fn getAddressingMode(As: u8) -> AddressingMode {
    match As {
        0b00 => Direct,
        0b10 => Indirect,
        0b11 => IndirectInc,
        0b01 => Indexed,
        _ => fail!("Invalid addressing mode")
    }
}

impl Cpu {

    // memory/register interface

    fn load(&mut self, regadr: u8, mode: AddressingMode) -> u16 {
        let regval = self.regs.load(regadr);
        match mode {
            Direct => regval,
            Indirect => self.ram.load(regval, self.inst.bw),
            IndirectInc => {
                self.regs.store(regadr, regval + 1);
                self.ram.load(regval, self.inst.bw)
            }
            Indexed => {
                let offset = self.next_inst();
                self.ram.load(regval + offset, self.inst.bw)
            }
        }
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
            Indexed => {
                let offset = self.next_inst();
                self.ram.store(regval + offset, val, self.inst.bw )
            }
        }
    }

    fn store(&mut self, val: u16) {
        self._store(self.inst.destreg, self.inst.Ad, val)
    }

    fn set_and_store(&mut self, val: u16) {
        self.setflags(val);
        self.store(val);
    }

    fn caller(&mut self) {
        match self.inst.optype {
            NoArg => match self.inst.opcode {
                0b000 => self.JNE(),
                0b001 => self.JEQ(),
                0b010 => self.JNC(),
                0b011 => self.JC(),
                0b100 => self.JN(),
                0b101 => self.JGE(),
                0b110 => self.JL(),
                0b111 => self.JMP(),
                _ => fail!("Illegal opcode")
                },
            OneArg => match self.inst.opcode {
                0b000 => self.RRC(),
                0b001 => self.SWPB(),
                0b010 => self.RRA(),
                0b011 => self.SXT(),
                0b100 => self.PUSH(),
                0b101 => self.CALL(),
                0b110 => self.RETI(),
                _ => fail!("Illegal opcode")
                },
            TwoArg => match self.inst.opcode {
                0b0100 => self.MOV(),
                0b0101 => self.ADD(),
                0b0110 => self.ADDC(),
                0b0111 => self.SUBC(),
                0b1001 => self.SUB(),
                0b1010 => self.DADD(),
                0b1011 => self.BIT(),
                0b1100 => self.BIC(),
                0b1101 => self.BIS(),
                0b1110 => self.XOR(),
                0b1111 => self.AND(),
                _ => fail!("Illegal opcode")
            }
        }
    }

    // utility functions

    fn getflag(self, flag: u16) -> bool {
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
        inst
    }

    // load and execute one instruction
    fn step(&mut self) { 
        let code = self.next_inst();
        self.inst = parse_inst(code);
        self.caller()
    }


    //Instructions

    // No args
    // TODO: These calls should use the API

    fn JNE(&mut self) {
        if !self.getflag(ZEROF) {
           self.regs.arr[0] = self.regs.arr[0] + self.inst.offset
        }
    }

    fn JEQ(&mut self) {
        if self.getflag(ZEROF) {
           self.regs.arr[0] = self.regs.arr[0] + self.inst.offset
        }
    }

    fn JNC(&mut self) {
        if !self.getflag(CARRYF) {
           self.regs.arr[0] = self.regs.arr[0] + self.inst.offset
        }
    }

    fn JC(&mut self) {
        if !self.getflag(CARRYF) {
           self.regs.arr[0] = self.regs.arr[0] + self.inst.offset
        }
    }

    fn JN(&mut self) {
        if self.getflag(NEGF) {
           self.regs.arr[0] = self.regs.arr[0] + self.inst.offset
        }
    }

    fn JGE(&mut self) {
        if self.getflag(NEGF) == self.getflag(OVERF) {
           self.regs.arr[0] = self.regs.arr[0] + self.inst.offset
        }
    }

    fn JL(&mut self) {
        if !(self.getflag(NEGF) == self.getflag(OVERF)) {
           self.regs.arr[0] = self.regs.arr[0] + self.inst.offset
        }
    }

    fn JMP(&mut self) {
       self.regs.arr[0] = self.regs.arr[0] + self.inst.offset
    }

    // One arg

    fn RRC(&mut self) {
        //think this is wrong
        let mut val = self.load(self.inst.destreg, self.inst.Ad);
        let C = self.getflag(CARRYF);
        val >>= 1;
        if C { val |= 0x8000 }
        self.set_and_store(val)
    }

    fn SWPB(&mut self) {
        let val = self.load(self.inst.destreg, self.inst.Ad);
        let topbyte = val >> 8;
        let botbyte = val << 8;
        self.store(topbyte | botbyte)
    }

    fn RRA(&mut self) {
        // TODO: Implement
        fail!("Not implemented")
    }

    fn SXT(&mut self) {
        let mut val = self.load(self.inst.destreg, self.inst.Ad);
        if (val & 0x0080) != 0 {
            //negative
            val |= 0xff00
        } else {
            //positive
            val &= 0x00ff
        }
        self.store(val)
    }

    fn PUSH(&mut self) {
        let val = self.load(self.inst.destreg, self.inst.Ad);
        let sp = self.load(self.inst.destreg, self.inst.Ad);
        self._store(2u8, Indirect, val)
    }

    fn CALL(&mut self) {
    }

    fn RET(&mut self) {
    }

    fn RETI(&mut self) {
    }

    // Two arg

    fn MOV(&mut self) {
        let val = self.load(self.inst.sourcereg, self.inst.As);
        self.store(val)
    }

    fn ADD(&mut self) {
        let inc = self.load(self.inst.sourcereg, self.inst.As);
        let val = self.load(self.inst.destreg, self.inst.Ad);
        self.set_and_store(val + inc)
    }

    fn ADDC(&mut self) {
        let inc = self.load(self.inst.sourcereg, self.inst.As);
        let val = self.load(self.inst.destreg, self.inst.Ad);
        let C = self.getflag(CARRYF);
        if C {
            self.set_and_store(val + inc + 1)
        } else {
            self.set_and_store(val + inc)
        }
    }

    fn SUBC(&mut self) {
        let inc = self.load(self.inst.sourcereg, self.inst.As);
        let val = self.load(self.inst.destreg, self.inst.Ad);
        let C = self.getflag(CARRYF);
        if C {
            self.set_and_store(val - inc + 1)
        } else {
            self.set_and_store(val - inc)
        }
    }

    fn SUB(&mut self) {
        let inc = self.load(self.inst.sourcereg, self.inst.As);
        let val = self.load(self.inst.destreg, self.inst.Ad);
        self.set_and_store(val - inc)
    }

    fn CMP(&mut self) {
        let inc = self.load(self.inst.sourcereg, self.inst.As);
        let val = self.load(self.inst.destreg, self.inst.Ad);
        self.setflags(val - inc);
    }

    fn DADD(&mut self) {
        fail!("Not implemented")
    }

    fn BIT(&mut self) {
        let inc = self.load(self.inst.sourcereg, self.inst.As);
        let val = self.load(self.inst.destreg, self.inst.Ad);
        self.setflags(inc & val);
    }

    fn BIC(&mut self) {
        let inc = self.load(self.inst.sourcereg, self.inst.As);
        let val = self.load(self.inst.destreg, self.inst.Ad);
        self.store(val & !inc)
    }

    fn BIS(&mut self) {
        let inc = self.load(self.inst.sourcereg, self.inst.As);
        let val = self.load(self.inst.destreg, self.inst.Ad);
        self.store(val | inc)
    }

    fn XOR(&mut self) {
        let inc = self.load(self.inst.sourcereg, self.inst.As);
        let val = self.load(self.inst.destreg, self.inst.Ad);
        self.set_and_store(val ^ inc)
    }

    fn AND(&mut self) {
        let inc = self.load(self.inst.sourcereg, self.inst.As);
        let val = self.load(self.inst.destreg, self.inst.Ad);
        self.set_and_store(val & inc)
    }

    fn new() -> Cpu { 
        Cpu {
            regs: Regs::new(),
            ram: Ram::new(),
            inst: Instruction::new()
        }
    }
}

#[test]
// Add a bunch of tests here. Important to get right.
fn parse_tests() {
    let instrs: [u16,..1] =         [0x4031]; //MOV
    let optype: [OpType,..1]=       [TwoArg];
    let opcodes: [u8,..1]=          [0b0100];
    let sourceregs: [u8,..1]=       [0b0000];
    let Ads: [AddressingMode,..1] = [Direct];
    let bws: [bool,..1] =           [false];
    let Ass: [AddressingMode,..1] = [Direct];
    let destregs: [u8,..1] =        [0b0001];
    for (ix, &code) in instrs.iter().enumerate() {
        let inst = parse_inst(code);
        assert_eq!(inst.opcode, opcodes[ix]);
        assert_eq!(inst.optype as u8, optype[ix] as u8);
        assert_eq!(inst.sourcereg, sourceregs[ix]);
        assert_eq!(inst.Ad as u8, Ads[ix] as u8);
        assert_eq!(inst.bw, bws[ix]);
        assert_eq!(inst.As as u8, Ass[ix] as u8);
        assert_eq!(inst.destreg, destregs[ix]);
    }
}

#[test]
fn cpu_test() {
    let mut cpu = Cpu::new();
}
