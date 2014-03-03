VPATH = src
OPTFLAGS = "-O"
NCURSESDIR = "lib/ncurses-rs/lib/"
NCURSESLIB = "libcurses-f5aa8b14-5.71.rlib"
HEXS = $(wildcard images/*.hex)
ALLBINS =  $(patsubst %.hex,%.bin,$(HEXS))

all: uctf

uctf: main.rs gui.rs cpu.rs mem.rs 
	rustc $< -o $@ -L $(NCURSESDIR)

uctfopt: main.rs gui.rs cpu.rs mem.rs 
	rustc $< -o $@ -L $(NCURSESDIR) $(OPTFLAGS)

dep: $(NCURSESDIR)/$(NCURSESLIB)

$(NCURSESDIR)/$(NCURSESLIB): lib/ncurses-rs
	make -C lib/ncurses-rs/ .build_lib

lib/ncurses-rs:
	mkdir -p lib
	git clone https://github.com/jeaye/ncurses-rs.git lib/ncurses-rs

clean: 
	rm -f uctf uctfopt
	rm -f images/*.bin

buildbins: $(ALLBINS)

%.bin : %.hex
	tools/makebin.py $*.hex

distclean: clean
	rm -rf lib

.PHONY: all dep clean distclean buildbins
