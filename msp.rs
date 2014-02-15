/*
000 RRC(.B) 9-bit rotate right through carry. C->msbit->...->lsbit->C. Clear the carry bit beforehand to do a logical right shift.
001 SWPB    Swap 8-bit register halves. No byte form.
010 RRA(.B) Badly named, this is an 8-bit arithmetic right shift.
011 SXT Sign extend 8 bits to 16. No byte form.
100 PUSH(.B)    Push operand on stack. Push byte decrements SP by 2. CPU BUG: PUSH #4 and PUSH #8 do not work when the short encoding using @r2 and @r2+ is used. The workaround, to use a 16-bit immediate, is trivial, so TI do not plan to fix this bug.
101 CALL    Fetch operand, push PC, then assign operand value to PC. Note the immediate form is the most commonly used. There is no easy way to perform a PC-relative call; the PC-relative addressing mode fetches a word and uses it as an absolute address. This has no byte form.
110 RETI    Pop SP, then pop PC. Note that because flags like CPUOFF are in the stored status register, the CPU will normally return to the low-power mode it was previously in. This can be changed by adjusting the SR value stored on the stack before invoking RETI (see below). The operand field is unused.
111 Not used    The MSP430 actually only has 27 instructions.

0100    MOV src,dest    dest = src  The status flags are NOT set.
0101    ADD src,dest    dest += src  
0110    ADDC src,dest   dest += src + C  
0111    SUBC src,dest   dest += ~src + C     
1001    SUB src,dest    dest -= src Implemented as dest += ~src + 1.
1001    CMP src,dest    dest - src  Sets status only; the destination is not written.
1010    DADD src,dest   dest += src + C, BCD.    
1011    BIT src,dest    dest & src  Sets status only; the destination is not written.
1100    BIC src,dest    dest &= ~src    The status flags are NOT set.
1101    BIS src,dest    dest |= src The status flags are NOT set.
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

enum OP {
    MOV,
    ADD,
    ADDC,
    SUBC,
    SUB,
    CMP,
    DADD,
    BIT,
    BIC,
    BIS,
    XOR,
    AND,
    JNE,
    JEQ,
    JNC,
    JC,
    JN, 
    JGE, 
    JL, 
    JMP,
    RRC,
    SWPB,
    RRA,
    SXT,
    PUSH,
    CALL,
    RETI
}

enum Opformat {
    OneArg,
    Jump,
    TwoArg
}

fn opformat(inst: u16) -> Opformat {
    if inst & 0xc000 > 0 {
        return OneArg
    } else if inst & 0x2000 > 0 {
        return Jump
    } else if inst & 0x1000 > 0 {
        return TwoArg
    } else {
        fail!("Decode failed at instruction: {:x}", inst)
    }
}

fn getOpcode(inst: u16, opfmt: Opformat) -> u8 {
    let op = match opfmt {
        OneArg => (inst & 0x0380)>>6,
        TwoArg => (inst & 0xf000)>>11,
        _ => 0,
    };
    return op as u8
}

fn main() {
    for &num in [0x8000, 0x4f00, 0x20f0,0x100f].iter() {
        let form = opformat(num);
        let op = getOpcode(num, form);
        println!("{:04t}", op)
    }
}

    



