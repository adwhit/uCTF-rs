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

type Cycles = u64;

type Regs = [u16, ..15];

pub struct Ram([u8, ..0x10000]);

struct Cpu<M> {
    cy: Cycles,
    regs: Regs,
    mem: M,
}

fn caller(inst: u16) {
    match opformat(inst) {
        NoArg => noarg_caller(inst),
        OneArg => onearg_caller(inst),
        TwoArg => twoarg_caller(inst),
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


fn noarg_caller(inst: u16) {
    match noarg_split(inst) {
    (0b000, offset) => JNE(offset),
    (0b001, offset) => JEQ(offset),
    (0b010, offset) => JNC(offset),
    (0b011, offset) => JC(offset),
    (0b100, offset) => JN(offset),
    (0b101, offset) => JGE(offset),
    (0b110, offset) => JL(offset),
    (0b111, offset) => JMP(offset),
    (_, offset) => fail!("Illegal match in noarg")
    }
}

fn onearg_caller(inst: u16) {
    match onearg_split(inst) {
    (0b000, bw, Ad, destreg) => RRC(bw,Ad,destreg),
    (0b001, bw, Ad, destreg) => SWPB(bw,Ad,destreg),
    (0b010, bw, Ad, destreg) => RRA(bw,Ad,destreg),
    (0b011, bw, Ad, destreg) => SXT(bw,Ad,destreg),
    (0b100, bw, Ad, destreg) => PUSH(bw,Ad,destreg),
    (0b101, bw, Ad, destreg) => CALL(bw,Ad,destreg),
    (0b110, bw, Ad, destreg) => RETI(bw,Ad,destreg),
    (_, bw, Ad, destreg) => fail!("Illegal match in onearg")
    }
}

fn twoarg_caller(inst: u16) {
    match twoarg_split(inst) {
    (0100, sourcereg, Ad, bw, As, destreg) => MOV(sourcereg, Ad, bw, As, destreg),
    (0101, sourcereg, Ad, bw, As, destreg) => ADD(sourcereg, Ad, bw, As, destreg),
    (0110, sourcereg, Ad, bw, As, destreg) => ADDC(sourcereg, Ad, bw, As, destreg),
    (0111, sourcereg, Ad, bw, As, destreg) => SUBC(sourcereg, Ad, bw, As, destreg),
    (1001, sourcereg, Ad, bw, As, destreg) => SUB(sourcereg, Ad, bw, As, destreg),
    (1010, sourcereg, Ad, bw, As, destreg) => DADD(sourcereg, Ad, bw, As, destreg),
    (1011, sourcereg, Ad, bw, As, destreg) => BIT(sourcereg, Ad, bw, As, destreg),
    (1100, sourcereg, Ad, bw, As, destreg) => BIC(sourcereg, Ad, bw, As, destreg),
    (1101, sourcereg, Ad, bw, As, destreg) => BIS(sourcereg, Ad, bw, As, destreg),
    (1110, sourcereg, Ad, bw, As, destreg) => XOR(sourcereg, Ad, bw, As, destreg),
    (1111, sourcereg, Ad, bw, As, destreg) => AND(sourcereg, Ad, bw, As, destreg),
    (_,_,_,_,_,_) => fail!("Illegal match in twoarg")
    }
}

// Operations

fn MOV() {
}

fn ADD() {
}

fn ADDC() {
}

fn SUBC() {
}

fn SUB() {
}

fn CMP() {
}

fn DADD() {
}

fn BIT() {
}

fn BIC() {
}

fn BIS() {
}

fn XOR() {
}

fn AND() {
}

fn JNE() {
}

fn JEQ() {
}

fn JNC() {
}

fn JC() {
}

fn JN() {
}

fn JGE() {
}

fn JL() {
}

fn JMP() {
}

fn RRC() {
}

fn SWPB() {
}

fn RRA() {
}

fn SXT() {
}

fn PUSH() {
}

fn CALL() {
}

fn RET() {
}

fn RETI() {
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
