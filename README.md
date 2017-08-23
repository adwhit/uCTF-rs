uCTF-rs
===

An msp430 emulator/debugger written in rust. It implements the interrupt features necessary to run the [microcorruption](www.microcorruption.com) capture-the-flag.

### Usage

To build, first make sure you are running the lastest build of rust (try the [rust-nightly](https://launchpad.net/~hansjorg/+archive/rust) repos), then run
```
make dep
make
```

The ```images``` folder contains hex dumps of the microcorruption levels. To compile them into executables, run
```
make buildbins
```

To execute the programs, run
```
./uctf images/IMAGE.bin
```

The first two levels are provided, - you will have to unlock the rest!

Once inside the debugger, use s, c, f, r, b, d and q to navigate.

The -d flag will dump the disassembled programme instructions to stdout and exit.

### What does it look like?

![uCTF](tools/uCTF.png)

### Why does this exist?

It's a [hackerschool](https://www.hackerschool.com) project I made to learn about rust, assembly, emulators and ncurses.

### Why does this STILL exist? (Update 2017-08-03)

I made this a long time ago and have thought about fixing it to compile on modern Rust,
but I think I prefer it as a time-capsule. The tildes! They're so beautiful!
