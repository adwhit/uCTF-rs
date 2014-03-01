FLAGS = -O

all:
	rustc src/main.rs -o msp -L lib/ncurses-rs/lib/

opt:
	rustc src/main.rs -o msp -L lib/ncurses-rs/lib/ $(FLAGS)
