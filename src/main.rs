#[feature(globs)];

extern crate ncurses;

use cpu::Cpu;
use std::io::File;
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
    loop {
        match nc::wgetch(nc::stdscr) {
            115 => {                //s
                cpu.step(); 
                windows.render(&cpu);
            },
            99 => loop {            //c
                cpu.step(); 
                windows.render(&cpu);
            },
            113 => break,           //q
            _ => ()
            }
    }
    nc::endwin();
}
