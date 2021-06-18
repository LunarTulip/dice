all: test

test: /target/debug/cli.exe
	./target/debug/cli.exe "(2 + 3   ) * (24d(((9))+-(2))) + (5d5)"

/target/debug/cli.exe:
	cargo build --bin cli
