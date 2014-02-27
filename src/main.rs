#[feature(globs)];

extern crate ncurses;

use cpu::{Cpu, GetInput, Normal, Success, Off};
use std::io::{File, stdin};
use std::os::args;
use nc = ncurses;

mod cpu;
mod gui;
mod mem;

fn main() {
    let argv = args();
    let fpath = argv[1];
    let memval = File::open(&Path::new(fpath)).read_to_end();
    let v = match memval {
        Ok(v) => v,
        Err(e) => fail!(e)
    };
    let mut cpu = Cpu::init(v);
    let mut windows = gui::Gui::init();
    windows.render(&cpu);
    let mut breakpoints : ~[u16] = ~[];
    'main : loop {
        match nc::wgetch(nc::stdscr) {
            115 => {                //s
                cpu.step();
                match &cpu.status {
                    &Off => {
                        cpu.buf.push_str("CPU OFF\n");
                        windows.render(&cpu);
                    },
                    &Success => {
                        cpu.buf.push_str("Success! Door unlocked"); 
                        windows.render(&cpu);
                    },
                    &GetInput(_) => { cpu.status = GetInput(windows.getstring()) },
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
                        &GetInput(_) => { cpu.status = GetInput(windows.getstring()); break 'outer },
                        &Normal => if c == 99 {windows.render(&cpu)},
                    }
                    for &num in breakpoints.iter() { if cpu.inst.memloc == num { break 'outer } }
                }
            },
            113 => break 'main,
            98 => {                 //b  -> breakpoint
                let s = windows.getstring();
                let noption = std::u16::parse_bytes(s.into_bytes(), 16);
                match noption {
                    Some(n) => {
                        breakpoints.push(n);
                        cpu.buf.push_str(format!("Breakpoint added: {:04x}\n", n));
                        windows.render(&cpu);
                    },
                    None => ()
                }
            },
            114 => {                //r 
                cpu = Cpu::init(v);
                windows = gui::Gui::init();
                cpu.buf.push_str("CPU reset\n"); 
                windows.render(&cpu); 
            },
            _ => ()
        }
    }
    nc::endwin();
}

