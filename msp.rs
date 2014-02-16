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
}

impl<M:Mem> MemUtil for M {
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

    fn store(&self, addr: u16, val: u16, byteflag: bool) -> u16 {
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
    fn load(&self, addr: u16) -> u16 {
        self.arr[addr]
    }
    fn store(&mut self, addr: u16, val: u16) {
        self.arr[addr] = val
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

fn get_optype(inst: u16) -> OpType {
    match inst >> 12 {
        0 => NoArg,
        1 => OneArg,
        _ => TwoArg
}

//splitters

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
        bw : bw
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
        bw : bw
    }
}

fn noarg_split(inst: u16) -> Instruction {
    let offset = (inst & 0x1ff);
    let opcode = ((inst & 0x1c00) >> 7) as u8;
    Instruction { 
        code: inst,
        optype: NoArg,
        opcode: opcode,
        offset: offset
    }
}

fn getAddressingMode(As: u8) -> Mode {
    match As {
        0b00 => Direct,
        0b10 => Indirect,
        0b11 => IndirectInc,
        0b01 => Indexed
    }
}

impl Cpu {

    fn load(&mut self, regadr: u16, mode: AddressingMode) -> u16 {
        let regval = self.regs.load(regadr);
        match mode {
            Direct => regval,
            Indirect => self.ram.load(regval),
            IndirectInc => {
                self.regs.store(regadr, regval + 1);
                self.ram.load(regval),
            }
            Indexed => self.ram.load(regval + self.next_inst())
        }
    }

    fn store(&mut self, regadr: u16, mode: AddressingMode, val: u16) {
        let regval = self.regs.load(regadr);
        match mode {
            Direct => self.regs.store(regadr, val),
            Indirect => self.ram.store(regval, val),
            IndirectInc => {
                self.regs.store(regadr, regval + 1);
                self.ram.store(regval, val),
            }
            Indexed => self.ram.store(regval + self.next_inst(), val)
        }
    }

    fn store(&mut self) -> u16 {
        let src = match self.inst.optype {
            OneArg => self.inst.destreg,
            TwoArg => self.inst.sourcereg,
            _ => fail!("Invalid load")
        }
        let regval = self.regs.load(src)
        match self.inst.mode {
            Direct => regval,
            Indirect => self.ram.load(regval),
            IndirectInc => {
                self.regs.store(src, regval + 1);
                self.ram.load(regval),
            }
            Indexed => self.ram.load(regval + self.next_inst())
        }
    }

    fn caller(&mut self) {
        match opformat(inst) {
            NoArg => self.noarg_caller(cpu, inst),
            OneArg => self.onearg_caller(cpu, inst),
            TwoArg => self.twoarg_caller(cpu, inst),
        }
    }


    fn noarg_caller(&mut self) {
        match cpu.inst.opcode {
            0b000 => self.JNE(),
            0b001 => self.JEQ(),
            0b010 => self.JNC(),
            0b011 => self.JC(),
            0b100 => self.JN(),
            0b101 => self.JGE(),
            0b110 => self.JL(),
            0b111 => self.JMP(),
            _ => fail!("Illegal match in noarg")
        }
    }

    fn onearg_caller(&mut self) {
        match onearg_split(inst) {
            0b000 => self.RRC(),
            0b001 => self.SWPB(),
            0b010 => self.RRA(),
            0b011 => self.SXT(),
            0b100 => self.PUSH(),
            0b101 => self.CALL(),
            0b110 => self.RETI(),
            _ => fail!("Illegal match in onearg")
        }
    }

    fn twoarg_caller(&mut self) {
        match twoarg_split(inst) {
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
            _ => fail!("Illegal match in twoarg")
        }
    }

    // utility functions
    fn set_flag(&mut self, flag: u16, on: bool ) {
        if on {
            self.regs[2] = self.regs[2] | flag
        } else {
            self.regs[2] = self.regs[2] & !flag
        }
    }

    fn set_zn(&mut self, val: u16) -> u16 {
        self.set_flag(ZEROF, val == 0);
        self.set_flag(NEGF, val & 0x8000 != 0);
        val
    }

    fn next_inst(&mut self) -> u16 {
        let inst = self.loadw(self.regs[0]);
        self.regs[0] += 2;
        inst
    }

    //instructions
    // No args

    fn JNE(&mut self, offset: u16) {
        if (self.regs[2] & ZEROF) != 0 {
           self.regs[0] = self.regs[0] + offset
        }
    }

    fn JEQ(&mut self, offset: u16) {
        if (self.regs[2] & ZEROF) == 0 {
           self.regs[0] = self.regs[0] + offset
        }
    }

    fn JNC(&mut self, offset: u16) {
        if (self.regs[2] & CARRYF) == 0 {
           self.regs[0] = self.regs[0] + offset
        }
    }

    fn JC(&mut self, offset: u16) {
        if (self.regs[2] & CARRYF) != 0 {
           self.regs[0] = self.regs[0] + offset
        }
    }

    fn JN(&mut self, offset: u16) {
        if (self.regs[2] & NEGF) != 0 {
           self.regs[0] = self.regs[0] + offset
        }
    }

    fn JGE(&mut self, offset: u16) {
        if (self.regs[2] & NEGF) == ((self.regs[2] & OVERF) >> 6)  {
           self.regs[0] = self.regs[0] + offset
        }
    }

    fn JL(&mut self, offset: u16) {
        if (self.regs[2] & NEGF) != ((self.regs[2] & OVERF) >> 6)  {
           self.regs[0] = self.regs[0] + offset
        }
    }

    fn JMP(&mut self, offset: u16) {
       self.regs[0] = self.regs[0] + offset
    }

    // One arg

    fn RRC(&mut self, bw: bool, Ad: u8, dest: u8) {
    }

    fn SWPB(&mut self, bw: bool, Ad: u8, dest: u8) {
    }

    fn RRA(&mut self, bw: bool, Ad: u8, dest: u8) {
    }

    fn SXT(&mut self, bw: bool, Ad: u8, dest: u8) {
    }

    fn PUSH(&mut self, bw: bool, Ad: u8, dest: u8) {
    }

    fn CALL(&mut self, bw: bool, Ad: u8, dest: u8) {
    }

    fn RET(&mut self, bw: bool, Ad: u8, dest: u8) {
    }

    fn RETI(&mut self, bw: bool, Ad: u8, dest: u8) {
    }

    // Two arg

    fn MOV(&mut self, src: u8, Ad: bool, bw: bool, As: u8, dest: u8) {
    }

    fn ADD(&mut self, src: u8, Ad: bool, bw: bool, As: u8, dest: u8) {
    }

    fn ADDC(&mut self, src: u8, Ad: bool, bw: bool, As: u8, dest: u8) {
    }

    fn SUBC(&mut self, src: u8, Ad: bool, bw: bool, As: u8, dest: u8) {
    }

    fn SUB(&mut self, src: u8, Ad: bool, bw: bool, As: u8, dest: u8) {
    }

    fn CMP(&mut self, src: u8, Ad: bool, bw: bool, As: u8, dest: u8) {
    }

    fn DADD(&mut self, src: u8, Ad: bool, bw: bool, As: u8, dest: u8) {
    }

    fn BIT(&mut self, src: u8, Ad: bool, bw: bool, As: u8, dest: u8) {
    }

    fn BIC(&mut self, src: u8, Ad: bool, bw: bool, As: u8, dest: u8) {
    }

    fn BIS(&mut self, src: u8, Ad: bool, bw: bool, As: u8, dest: u8) {
    }

    fn XOR(&mut self, src: u8, Ad: bool, bw: bool, As: u8, dest: u8) {
    }

    fn AND(&mut self, src: u8, Ad: bool, bw: bool, As: u8, dest: u8) {
    }

    fn step(&mut self) {
        let instr = self.regs[0];
        caller(self, instr)

    }

    fn new() -> Cpu { 
        Cpu {regs: [0, ..15], ram: [0, ..0x10000]}
    } 
}

#[test]
fn opformat_test() {
    let nums: [u16,..3] = [0x4031, 0x2407, 0x12b0]; //mov, jz, call
    let opfmtres: [Opformat,..3] = [TwoArg, NoArg, OneArg];
    for (&num, &opfrm) in nums.iter().zip(opfmtres.iter()) {
        let form = opformat(num);
        assert_eq!(form as int, opfrm as int);
    }
}

#[test]
fn twoarg_split_test() {
    let instrs: [u16,..1] =    [0x4031]; //MOV
    let opcodes: [u8,..1]=     [0b0100];
    let sourceregs: [u8,..1]=  [0b0000];
    let Ads: [bool,..1] =      [false];
    let bws: [bool,..1] =      [false];
    let Ass: [u8,..1] =        [0b11];
    let destregs: [u8,..1] =   [0b0001];
    for (ix, &inst) in instrs.iter().enumerate() {
        let (opcode, sourcereg, Ad, bw, As, destreg) = twoarg_split(inst);
        assert_eq!(opcode, opcodes[ix]);
        assert_eq!(sourcereg, sourceregs[ix]);
        assert_eq!(Ad, Ads[ix]);
        assert_eq!(bw, bws[ix]);
        assert_eq!(As, Ass[ix]);
        assert_eq!(destreg, destregs[ix]);
    }
}

#[test]
fn cpu_test() {
    let mut cpu = Cpu::new();
    println!("{:?}", cpu)
}

