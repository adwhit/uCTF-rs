uCTF-rs
===

An msp430 emulator/debugger written in rust. It implemnts the interrupt features necessary to run the [microcorruption](www.microcorruption.com) capture-the-flag.

### Usage

To build, first make sure you are running the lastest build of rust (try the [rust-nightly](https://launchpad.net/~hansjorg/+archive/rust) repos), then run
```./configure && make```

The ```images``` folder contains hex dumps of the microcorruption levels. To compile them into executables, run
```
tools/makebin.py images/IMAGE.hex
```

To execute the programs, run
```
./uctf images/IMAGE
```
Once inside the debugger, use s, c, f, r, b, d and q to navigate.

Run 
```
./uctf images/IMAGE -d
```
to dump the disassembled programme instructions to stdout.

### What does it look like?

![uCTF](tools/uCTF.png)

### It's buggy!

Please file an issue.

### Why does this exist?

It's a [hackerschool](https://www.hackerschool.com) project I made to learn about rust, assembly, emulation and ncurses.

