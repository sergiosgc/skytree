default: debug

debug:
	RUST_BACKTRACE=1 cargo run -- --config-file=skytree/skytree.ini

release:
	cargo build --release

watch:
	RUST_BACKTRACE=1 cargo watch --watch skytree/src --watch skytree/templates -x 'run -- --config-file=skytree/skytree.ini'
