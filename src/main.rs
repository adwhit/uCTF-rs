#[feature(globs)];

extern crate ncurses;

use cpu::Cpu;
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
    let mut cpu = match memval {
        Ok(v) => Cpu::init(v),
        Err(e) => fail!(e)
    };
    let windows = gui::Gui::init();
    windows.render(&cpu);
    let mut breakpoints : ~[u16] = ~[];
    loop {
        match nc::wgetch(nc::stdscr) {
            115 => {                //s
                cpu.step(); 
                windows.render(&cpu);
            },
            99 => {
                'outer : loop {            //c
                    cpu.step(); 
                    windows.render(&cpu);
                    for &num in breakpoints.iter() {
                        if cpu.inst.memloc == num { break 'outer }
                    }
                }
            },
            113 => break,           //q
            98 => {                 //b
                let s = windows.getstring();
                let nopt = std::u16::parse_bytes(s.into_bytes(), 16);
                match nopt {
                    Some(n) => {
                        breakpoints.push(n);
                        cpu.buf.push_str(format!("Breakpoint added: {:04x}\n", n));
                        windows.render(&cpu);
                    },
                    None => ()
                }
            },
            _ => ()
        }
    }
    nc::endwin();
}
