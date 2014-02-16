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

enum Opformat {
    NoArg,
    OneArg,
    TwoArg
}

//Flags

static CARRYF : u16 = 1;
static ZEROF : u16 = 1 << 1;
static NEGF : u16 = 1 << 2;
static OVERF : u16 = 1 << 8;

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
}

struct Cpu {
    regs: [u16, ..15],
    ram: [u8, ..0x10000]
}

impl Mem for Cpu {
    fn loadb(&self, addr: u16) -> u8 {
        self.ram[addr & 0x7ff]
    }
    fn storeb(&mut self, addr: u16, val: u8) {
        self.ram[addr & 0x7ff] = val
    }
}

fn caller(cpu: &mut Cpu, inst: u16) {
    match opformat(inst) {
        NoArg => noarg_caller(cpu, inst),
        OneArg => onearg_caller(cpu, inst),
        TwoArg => twoarg_caller(cpu, inst),
    }
}

fn opformat(inst: u16) -> Opformat {
    if inst & 0xc000 > 0 {
        return TwoArg
    } else if inst & 0x2000 > 0 {
        return NoArg
    } else if inst & 0x1000 > 0 {
        return OneArg
    } else {
        fail!("Decode failed at instruction: {:x}", inst)
    }
}

fn twoarg_split(inst: u16) -> (u8, u8, bool, bool, u8, u8) {
    let destreg = (inst & 0xf) as u8;
    let As = ((inst & 0x30) >> 4) as u8;
    let bw = ((inst & 0x40) >> 6) != 0;
    let Ad = ((inst & 0x80) >> 7) != 0;
    let sourcereg = ((inst & 0xf00) >> 8) as u8;
    let opcode = ((inst & 0xf000) >> 12) as u8;
    return (opcode, sourcereg, Ad, bw, As, destreg)
}

fn onearg_split(inst: u16) -> (u8, bool, u8, u8) {
    let destreg = (inst & 0xf) as u8;
    let Ad = ((inst & 0x30) >> 4) as u8;
    let bw = ((inst & 0x40) >> 6) == 0;
    let opcode = ((inst & 0x380) >> 7) as u8;
    return (opcode, bw, Ad, destreg)
}

fn noarg_split(inst: u16) -> (u8, u16) {
    let offset = (inst & 0x1ff);
    let opcode = ((inst & 0x1c00) >> 7) as u8;
    return (opcode, offset)
}


fn noarg_caller(cpu: &mut Cpu, inst: u16) {
    match noarg_split(inst) {
    (0b000, offset) => cpu.JNE(offset),
    (0b001, offset) => cpu.JEQ(offset),
    (0b010, offset) => cpu.JNC(offset),
    (0b011, offset) => cpu.JC(offset),
    (0b100, offset) => cpu.JN(offset),
    (0b101, offset) => cpu.JGE(offset),
    (0b110, offset) => cpu.JL(offset),
    (0b111, offset) => cpu.JMP(offset),
    (_, offset) => fail!("Illegal match in noarg")
    }
}

fn onearg_caller(cpu: &mut Cpu, inst: u16) {
    match onearg_split(inst) {
    (0b000, bw, Ad, destreg) => cpu.RRC(bw,Ad,destreg),
    (0b001, bw, Ad, destreg) => cpu.SWPB(bw,Ad,destreg),
    (0b010, bw, Ad, destreg) => cpu.RRA(bw,Ad,destreg),
    (0b011, bw, Ad, destreg) => cpu.SXT(bw,Ad,destreg),
    (0b100, bw, Ad, destreg) => cpu.PUSH(bw,Ad,destreg),
    (0b101, bw, Ad, destreg) => cpu.CALL(bw,Ad,destreg),
    (0b110, bw, Ad, destreg) => cpu.RETI(bw,Ad,destreg),
    (_, bw, Ad, destreg) => fail!("Illegal match in onearg")
    }
}

fn twoarg_caller(cpu: &mut Cpu, inst: u16) {
    match twoarg_split(inst) {
    (0b0100, sourcereg, Ad, bw, As, destreg) => cpu.MOV(sourcereg, Ad, bw, As, destreg),
    (0b0101, sourcereg, Ad, bw, As, destreg) => cpu.ADD(sourcereg, Ad, bw, As, destreg),
    (0b0110, sourcereg, Ad, bw, As, destreg) => cpu.ADDC(sourcereg, Ad, bw, As, destreg),
    (0b0111, sourcereg, Ad, bw, As, destreg) => cpu.SUBC(sourcereg, Ad, bw, As, destreg),
    (0b1001, sourcereg, Ad, bw, As, destreg) => cpu.SUB(sourcereg, Ad, bw, As, destreg),
    (0b1010, sourcereg, Ad, bw, As, destreg) => cpu.DADD(sourcereg, Ad, bw, As, destreg),
    (0b1011, sourcereg, Ad, bw, As, destreg) => cpu.BIT(sourcereg, Ad, bw, As, destreg),
    (0b1100, sourcereg, Ad, bw, As, destreg) => cpu.BIC(sourcereg, Ad, bw, As, destreg),
    (0b1101, sourcereg, Ad, bw, As, destreg) => cpu.BIS(sourcereg, Ad, bw, As, destreg),
    (0b1110, sourcereg, Ad, bw, As, destreg) => cpu.XOR(sourcereg, Ad, bw, As, destreg),
    (0b1111, sourcereg, Ad, bw, As, destreg) => cpu.AND(sourcereg, Ad, bw, As, destreg),
    (_,_,_,_,_,_) => fail!("Illegal match in twoarg")
    }
}

// Operations

impl Cpu {

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

