build:
	cargo build
run:
	cargo run -- -i test/small -o res.png
debug:
	cargo run -- -i test/small -d -dp debug -s 5
