all: /target/debug/dice.exe

test: /target/debug/dice.exe
	./target/debug/dice.exe "(2 + 3   ) * (24d(((9))+-(2))) + (5d5)"

/target/debug/dice.exe:
	cargo build --bin dice
