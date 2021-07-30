# All

all: target/debug/fluorite.exe target/debug/fluorite-gui.exe

# Development

gui: target/debug/fluorite-gui.exe
	./target/debug/fluorite-gui.exe

test: target/debug/fluorite.exe
	./target/debug/fluorite.exe -v "(2 + 3   ) * 24d(((9))+-(2)) + (5d5)"

target/debug/fluorite.exe: src/dice.pest src/lib.rs src/parse.rs src/bin/fluorite.rs
	cargo build --bin fluorite

target/debug/fluorite-gui.exe: src/dice.pest src/lib.rs src/parse.rs src/bin/fluorite-gui.rs
	cargo build --bin fluorite-gui

# Release

release-all: release-win64

release-win64: target/x86_64-pc-windows-msvc/release/fluorite.exe target/x86_64-pc-windows-msvc/release/fluorite-gui.exe
	mkdir -p release-staging/win64/Fluorite
	cp target/x86_64-pc-windows-msvc/release/fluorite.exe release-staging/win64/Fluorite/fluorite.exe
	cp target/x86_64-pc-windows-msvc/release/fluorite-gui.exe release-staging/win64/Fluorite/fluorite-gui.exe
	cd release-staging/win64 && zip -r ../fluorite-win64-$(shell cargo metadata --format-version=1 --no-deps | jq -r '.packages[] | select(.name == "fluorite") | .version').zip Fluorite

target/x86_64-pc-windows-msvc/release/fluorite.exe: src/dice.pest src/lib.rs src/parse.rs src/bin/fluorite.rs
	cargo build --release --target x86_64-pc-windows-msvc --bin fluorite

target/x86_64-pc-windows-msvc/release/fluorite-gui.exe: src/dice.pest src/lib.rs src/parse.rs src/bin/fluorite-gui.rs
	cargo build --release --target x86_64-pc-windows-msvc --bin fluorite-gui
