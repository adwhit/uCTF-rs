
#[feature(globs)];

extern crate ncurses;
use cpu::Cpu;
use std::io::File;
use std::os::args;
use ncurses::*;

mod cpu;
mod mem;

fn main() {
    let argv = args();
    let fpath = match argv {
        [_,v, ..] => v,
        _ => fail!("Please supply file argument")
    };
    let mem = File::open(&Path::new(fpath)).read_to_end();
    let mut cpu = match mem {
        Ok(v) => Cpu::init(v),
        Err(e) => fail!(e)
    };
    initscr();
    raw();
    draw(cpu);
    while true {
        match getch() {
            113 => break,
            115 => { cpu.step(); draw(cpu); },
            _ => draw(cpu)
            }
    }
    endwin();
}

fn draw(cpu: Cpu) {
    clear();
    printw(format!("{}", cpu));
    mvprintw(LINES - 1, 0, "S to advance, Q to exit");
    refresh();
}
    

