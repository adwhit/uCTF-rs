use msp::Cpu;
use std::io::File;
use std::os::args;

mod msp;
mod mem;

fn main() {
    let argv = args();
    let fpath = argv[1];
    let mem = File::open(&Path::new(fpath)).read_to_end();
    let mut cpu = match mem {
        Ok(v) => Cpu::init(v),
        Err(e) => fail!(e)
    };
    println!("{}", cpu);
    cpu.step();
    println!("{}", cpu);
    cpu.step();
    println!("{}", cpu);

}
