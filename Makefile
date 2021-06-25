all: /target/debug/dice.exe

gui: /target/debug/dice-gui.exe
	./target/debug/dice-gui.exe

test: /target/debug/dice.exe
	./target/debug/dice.exe -v "(2 + 3   ) * 24d(((9))+-(2)) + (5d5)"

/target/debug/dice-gui.exe:
	cargo build --bin dice-gui

/target/debug/dice.exe:
	cargo build --bin dice
