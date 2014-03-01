
use ncurses::*;
use mem;
use cpu;
use std;

static RAMHEIGHT : i32 = 60;
static RAMWIDTH : i32 = 49;
static RAMX : i32 = 1;
static RAMY : i32 = 1;
static REGHEIGHT : i32 = 7;
static REGWIDTH : i32 = 37;
static REGX : i32 = 52;
static REGY : i32 = 1;
static ASMHEIGHT : i32 = 7;
static ASMWIDTH : i32 = 70;
static ASMX : i32 = 52;
static ASMY : i32 = 10;
static DBGHEIGHT : i32 = 10;
static DBGWIDTH : i32 = 50;
static DBGX : i32 = 52;
static DBGY : i32 = 20;

pub struct Gui {
    ramwin : WINDOW,
    regwin : WINDOW,
    asmwin : WINDOW,
    dbgwin : WINDOW,
    lit: [bool,..16]
}

impl Gui {

    pub fn init() -> Gui {
        initscr();
        clear();
        raw();
        noecho();
        start_color();

        init_pair(1, 1, COLOR_WHITE);
        init_pair(2, 2, COLOR_WHITE);
        init_pair(3, 3, COLOR_WHITE);
        init_pair(4, 4, COLOR_WHITE);
        init_pair(5, 5, COLOR_WHITE);
        init_pair(6, 6, COLOR_WHITE);

        let ramwin = newwin(RAMHEIGHT, RAMWIDTH, RAMY, RAMX);
        let regwin = newwin(REGHEIGHT, REGWIDTH, REGY, REGX);
        let asmwin = newwin(ASMHEIGHT, ASMWIDTH, ASMY, ASMX);
        let dbgwin = newwin(DBGHEIGHT, DBGWIDTH, DBGY, DBGX);

        box_(ramwin, 0, 0);
        box_(regwin, 0, 0);
        box_(asmwin, 0, 0);
        box_(dbgwin, 0, 0);

        mvwprintw(ramwin,0, 10, "RAM");
        mvwprintw(regwin,0, 10, "Registers");
        mvwprintw(asmwin,0, 10, "Instructions");

        refresh();

        Gui {
            ramwin: ramwin,
            regwin: regwin,
            asmwin: asmwin,
            dbgwin: dbgwin,
            lit: [false,..16]
        }
    }

    fn draw_ram(&mut self, r: mem::Ram, regs: mem::Regs, pc: u16) {
        self.lit = [false,..16];
        let mut rowct = 1;
        let mut printlast = false;
        'rows: for row in std::iter::range(0, r.arr.len()/16) {
            let mut zero = true;
            for col in range(0, 16u) {
                if r.arr[row * 16 + col] != 0 { zero = false } 
            }
            match (printlast, zero) {
                (true,true) => { 
                    mvwprintw(self.ramwin, rowct, 1, "                 ***                          ");
                    printlast = false;
                    rowct += 1;
                    continue 'rows
                },
                (true, false) => (),
                (false, true) => {continue 'rows}
                (false,false) => { 
                    printlast = true;
                }
            }
            wmove(self.ramwin, rowct, 1); rowct += 1;
            wprintw(self.ramwin, format!("{:04x}:  ", row * 16));
            'cols : for col in range(0u, 16u) { 
                if col % 2 == 0 {
                    let celln = row * 16 + col;
                    for regn in range(0, 16) {
                        let regf = (regn % 6) as i16 + 1;
                        let regval = if regn == 0 { pc } else {regs.arr[regn] & 0xfffe};
                        if celln == regval as uint {
                        // print in colour
                            wattron(self.ramwin, COLOR_PAIR(regf));
                            wprintw(self.ramwin, format!("{:02x}{:02x} ", r.arr[celln], r.arr[celln + 1]));
                            self.lit[regn] = true;
                            wattroff(self.ramwin, COLOR_PAIR(regf));
                            continue 'cols;
                        }
                    }
                        // normal print
                    wprintw(self.ramwin, format!("{:02x}{:02x} ", r.arr[celln], r.arr[celln + 1]));
                }
            }
        }
        wrefresh(self.ramwin);
    }

    fn draw_regs(&self, r: mem::Regs, inst: cpu::Instruction) {
        let mut linect = 0;
        for regn in range(0, 16) {
            let s : ~str;
            if regn % 4 == 3 {
                s = format!("R{:02i} {:04x}", regn, r.arr[regn]);
            } else if regn == 0 { 
                s = format!("PC  {:04x} ", inst.memloc)
            } else {
                s = format!("R{:02i} {:04x} ", regn, r.arr[regn])
            }
            if regn % 4 == 0 { linect += 1; wmove(self.regwin, linect, 1);}
            if self.lit[regn] {
                colprint(self.regwin, (regn % 6 +1) as i16, s)
            } else {
                wprintw(self.regwin, s);
            }
        }
        mvwprintw(self.regwin, 5,10,format!("{:20s}", inst.to_string()));
        wrefresh(self.regwin);
    }

    fn draw_inst(&self, inst: cpu::Instruction) {
        mvwprintw(self.asmwin, 1,1, format!("MemLoc:0x{:04x} | Value:  0x{:04x}//{:016t}", 
                                            inst.memloc, inst.code,inst.code));
        mvwprintw(self.asmwin, 2,1, format!("OpType:{:06?} | Opcode:{:04t} | B/W:{:05b} | Offset: {:04x}",
                                            inst.optype, inst.opcode, inst.bw, inst.offset));
        mvwprintw(self.asmwin, 3,1, format!("DestReg:  {:02u}  | DestMode:  {} ", 
                                            inst.destreg, inst.destmode));
        mvwprintw(self.asmwin, 4,1,format!("SourceReg:{:02u}  | SourceMode:{} ",
                                           inst.srcreg, inst.srcmode));
        mvwprintw(self.asmwin, 5,1,format!("{:20s}", inst.to_string()));
        wrefresh(self.asmwin);
    }

    fn draw_debug(&self, s: &str) {
        let lines : ~[&str] = s.clone().lines().collect();
        let l = lines.len();
        let mut ct = 1;
        for (ix, &line) in lines.iter().enumerate() { 
            if (ix as i32) > (l as i32) - DBGHEIGHT + 1 {
                mvwprintw(self.dbgwin, ct, 1, line);
                ct +=1 
            }
        }
        wrefresh(self.dbgwin);
    }

    pub fn render(&mut self, cpu: &cpu::Cpu) {
        box_(self.ramwin, 0, 0);
        box_(self.regwin, 0, 0);
        box_(self.asmwin, 0, 0);
        box_(self.dbgwin, 0, 0);
        self.draw_ram(cpu.ram, cpu.regs, cpu.inst.memloc);
        self.draw_regs(cpu.regs, cpu.inst);
        self.draw_inst(cpu.inst);
        self.draw_debug(cpu.buf);
        mvprintw(LINES - 2, 0, "s: step, c: continue, f: fast-forward, b: add breakpoint, d to redraw, q: quit");
        move(LINES - 3, 10); // to allow println! usage
        refresh();
    }

    pub fn getstring(&self) -> ~str {
        let mut s = ~"";
        echo();
        wmove(self.dbgwin, DBGHEIGHT-2,2);
        wgetstr(self.dbgwin, &mut s);
        noecho();
        s
    }
}

fn colprint(win: WINDOW, col: i16, s: ~str) {
    wattron(win, COLOR_PAIR(col));
    wprintw(win, s);
    wattroff(win, COLOR_PAIR(col));
}

fn colmvprint(win: WINDOW, y:i32, x:i32, col: i16, s: ~str) {
    wattron(win, COLOR_PAIR(col));
    mvwprintw(win,y,x,s);
    wattroff(win, COLOR_PAIR(col));
}
