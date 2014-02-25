#[feature(globs)];

extern crate ncurses;

use cpu::Cpu;
pub use gui::mem;
use std::io::File;
use std::os::args;
use nc = ncurses;

mod cpu;
mod gui;

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
    
    loop {
        match nc::wgetch(nc::stdscr) {
            113 => break,
            115 => { 
                cpu.step(); 
                windows.draw_ram(cpu.ram, cpu.regs.arr[0]);
                windows.render();
            }
            _ => ()
            }
    }
    nc::endwin();
}
