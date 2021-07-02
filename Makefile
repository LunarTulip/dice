all: /target/debug/fluorite.exe

gui: /target/debug/fluorite-gui.exe
	./target/debug/fluorite-gui.exe

test: /target/debug/fluorite.exe
	./target/debug/fluorite.exe -v "(2 + 3   ) * 24d(((9))+-(2)) + (5d5)"

/target/debug/fluorite-gui.exe:
	cargo build --bin fluorite-gui

/target/debug/fluorite.exe:
	cargo build --bin fluorite
