default: debug

debug:
	RUST_LOG=debug RUST_BACKTRACE=1 DATABASE_URL=sqlite://database.sqlite cargo run -- --config-file=skytree/skytree.ini

release:
	cargo build --release

watch:
	RUST_LOG=debug RUST_BACKTRACE=1 DATABASE_URL=sqlite://database.sqlite cargo watch --watch skytree/src --watch skytree/templates --watch negotiated --watch rest --watch rest-derive -x 'run -- --config-file=skytree/skytree.ini'
