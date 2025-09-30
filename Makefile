default:
	cargo run --release
clean:
	rm -rf target `find . -name \*~`
