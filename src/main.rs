#[feature(globs)];

extern crate ncurses;
extern crate collections;
extern crate getopts;

use cpu::{Cpu, GetInput, Normal, Success, Off};
use std::io::{File, stdin};
use std::os;
use getopts::{optflag, getopts};
use nc = ncurses;

mod cpu;
mod gui;
mod mem;

fn print_usage(s: &str) {
    println!("Usage: {} [options] INPUT", s);
    println!("Options: -d --disasm      print disassembled input");
}

fn print_disasm(v: &[u8]) {
    let listing = cpu::disassemble(v);
    for (lineno, line) in listing.move_iter() {
        println!("{:04x}: {}", lineno, line)
    }
}

fn event_loop(mut cpu: Cpu, mut windows: gui::Gui, mut breakpoints: ~[u16]) -> (uint,~[u16]) {
    loop {
        match nc::wgetch(nc::stdscr) {
            115 => {                //s
                cpu.step();
                match &cpu.status {
                    &Off => {
                        cpu.buf.push_str("CPU OFF\n");
                        windows.render(&cpu);
                    },
                    &Success => {
                        cpu.buf.push_str("Success! Door unlocked\n"); 
                        windows.render(&cpu);
                    },
                    &GetInput(_) => { cpu.status = GetInput(str2bytes(getstring(cpu.buf))) },
                    &Normal => {windows.render(&cpu)},
                }
                windows.render(&cpu);
            }
            c @ 99 | c @ 102 => {
                'outer : loop {            //c or f
                    cpu.step();
                    match &cpu.status {
                        &Off => {
                            cpu.buf.push_str("CPU OFF\n");
                            windows.render(&cpu);
                            break 'outer
                        },
                        &Success => {
                            cpu.buf.push_str("Success! Door unlocked"); 
                            windows.render(&cpu);
                            break 'outer
                        },
                        &GetInput(_) => { 
                            windows.render(&cpu);
                            cpu.status = GetInput(str2bytes(getstring(cpu.buf)));
                            break 'outer 
                        },
                        &Normal => if c == 99 {windows.render(&cpu)},
                    }
                    for &num in breakpoints.iter() { 
                        if cpu.inst.memloc == num {
                            cpu.buf.push_str(format!("Break {:04x}\n", num)); windows.render(&cpu); break 'outer 
                        } 
                    }
                }
            },
            113 => return (1, ~[]),
            98 => {                 //b  -> breakpoint
                let s = getstring("Enter breakpoint location:\n");
                let noption = std::u16::parse_bytes(s.trim().to_owned().into_bytes(),16);
                match noption {
                    Some(n) => {
                        breakpoints.push(n & 0xfffe);
                        cpu.buf.push_str(format!("Breakpoint added: {:04x}\n", n & 0xfffe));
                        windows.render(&cpu);
                    },
                    None => cpu.buf.push_str(format!("Failed to add breakpoint {}\n", s.clone()))
                }
            },
            114 => return (0, breakpoints),               //r 
            100 => { nc::endwin(); windows.render(&cpu); nc::refresh(); },        //d
            _ => ()
        }
    }
}


fn main() {
    let args = os::args();
    let opts = ~[optflag("d", "disasm", "Print disassembled file")];
    let matches = match getopts(args.tail(), opts) {
        Ok(m) => m,
        Err(_) => { println!("Argument parse failed"); print_usage(args[0]); return }
    };
    let fpath = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        print_usage(args[0]);
        return;
    };
    let memval = File::open(&Path::new(fpath)).read_to_end();
    let v = match memval {
        Ok(v) => v,
        Err(e) => fail!(e)
    };
    if matches.opt_present("d") {
        print_disasm(v);
        return
    }


    let mut breakpoints : ~[u16] = ~[];
    let mut status = 0;
    while status == 0 {
        let cpu = Cpu::init(v);
        let mut windows = gui::Gui::init();
        windows.listing = cpu::disassemble(cpu.ram.arr);
        windows.render(&cpu);
        let (s, b) = event_loop(cpu, windows, breakpoints.clone());
        breakpoints = b;
        status = s;
        nc::endwin();
    }
}

fn str2bytes(s : &str) -> ~[u8] {
    let mut out = if s.starts_with(&'static "x") {
        let mut res: ~[u8] = ~[];
        let bytes :~[u8] = s.slice_from(1).bytes().collect();
        for chunk in bytes.chunks(2) {
            match std::u8::parse_bytes(chunk, 16) {
                Some(n) => res.push(n),
                None => ()
            }
        }
        res.push(0u8);
        res
    } else {
        s.bytes().collect()
    };
    out.push(0u8);
    out
}

fn getstring(buf: &str) -> ~str {
    nc::endwin();
    print!("{}", buf);
    let mut std = stdin();
    let s = std.read_line().unwrap();
    nc::refresh();
    s
}

