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
    let fpath = match argv {
        [_,v, ..] => v,
        _ => fail!("Please supply file argument")
    };
    let memval = File::open(&Path::new(fpath)).read_to_end();
    let mut cpu = match memval {
        Ok(v) => Cpu::init(v),
        Err(e) => fail!(e)
    };
    let windows = gui::Gui::init();
    windows.render(&cpu);
    loop {
        match nc::wgetch(nc::stdscr) {
            113 => break,
            115 => { 
                cpu.step(); 
                windows.render(&cpu);
            }
            _ => ()
            }
    }
    nc::endwin();
}
