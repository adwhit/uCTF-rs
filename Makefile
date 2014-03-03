FLAGS = -O

all:
	rustc src/main.rs -o uctf -L lib/ncurses-rs/lib/

opt:
	rustc src/main.rs -o uctf -L lib/ncurses-rs/lib/ $(FLAGS)
