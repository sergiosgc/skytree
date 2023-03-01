default: debug

debug:
	RUST_BACKTRACE=1 cargo run -- --config-file=skytree.ini

release:
	cargo build --release

watch:
	RUST_BACKTRACE=1 cargo watch -x 'run -- --config-file=skytree.ini'
