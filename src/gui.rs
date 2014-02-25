
use ncurses::*;
pub mod mem;

static RAMHEIGHT : i32 = 50;
static RAMWIDTH : i32 = 30;
static RAMX : i32 = 1;
static RAMY : i32 = 1;
static REGHEIGHT : i32 = 50;
static REGWIDTH : i32 = 30;
static REGX : i32 = 0;
static REGY : i32 = 32;
static ASMHEIGHT : i32 = 50;
static ASMWIDTH : i32 = 30;
static ASMX : i32 = 52;
static ASMY : i32 = 1;
static DBGHEIGHT : i32 = 50;
static DBGWIDTH : i32 = 30;
static DBGX : i32 = 52;
static DBGY : i32 = 30;

pub struct Gui {
    ramwin : WINDOW,
    regwin : WINDOW,
    asmwin : WINDOW,
    dbgwin : WINDOW,
}

impl Gui {

    pub fn init() -> Gui {
        initscr();
        clear();
        raw();
        noecho();
        start_color();
        init_pair(1, COLOR_RED, COLOR_WHITE);

        let ramwin = newwin(RAMHEIGHT, RAMWIDTH, RAMY, RAMX);
        let regwin = newwin(REGHEIGHT, REGWIDTH, REGY, REGX);
        let asmwin = newwin(ASMHEIGHT, ASMWIDTH, ASMY, ASMX);
        let dbgwin = newwin(DBGHEIGHT, DBGWIDTH, DBGY, DBGX);

        Gui {
            ramwin: ramwin,
            regwin: regwin,
            asmwin: asmwin,
            dbgwin: dbgwin
        }
    }

    pub fn draw_ram(&self, r: mem::Ram, pc: u16) {
        box_(self.ramwin, 0, 0);
        wmove(self.ramwin,1,1);
        for ix in range(0, r.arr.len()) {
            if ix % 2 == 0 {
                // take two at once
                if ix == pc as uint {
                    // print in colour
                    wattron(self.ramwin, COLOR_PAIR(1));
                    wprintw(self.ramwin, format!("{:02u}{:02u} ", r.arr[ix], r.arr[ix+1]));
                    wattroff(self.ramwin, COLOR_PAIR(1));
                } else {
                    // normal print
                    wprintw(self.ramwin, format!("{:u}{:u} ", r.arr[ix], r.arr[ix+1]));
                }
            }
            if ix % 16 == 0 { wmove(self.ramwin, 1, (ix/16) as i32); }
        }
        wrefresh(self.ramwin);
    }

    pub fn render(&self) {
        mvprintw(LINES - 1, 0, "S to advance, Q to exit");
        move(10, 10);
        refresh();
    }
}
