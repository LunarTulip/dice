all: target/debug/fluorite.exe target/debug/fluorite-gui.exe

release:
	cargo build --release --bin fluorite
	cargo build --release --bin fluorite-gui

gui: target/debug/fluorite-gui.exe
	./target/debug/fluorite-gui.exe

test: target/debug/fluorite.exe
	./target/debug/fluorite.exe -v "(2 + 3   ) * 24d(((9))+-(2)) + (5d5)"

target/debug/fluorite-gui.exe: src/dice.pest src/lib.rs src/parse.rs src/bin/fluorite-gui.rs
	cargo build --bin fluorite-gui

target/debug/fluorite.exe: src/dice.pest src/lib.rs src/parse.rs src/bin/fluorite.rs
	cargo build --bin fluorite
