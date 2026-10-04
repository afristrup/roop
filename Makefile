.PHONY: install

CARGO_BIN := $(or $(CARGO_INSTALL_ROOT),$(CARGO_HOME),$(HOME)/.cargo)/bin

install:
	cargo install --path crates/roop
	cargo build --release -p roop-rt
	cp target/release/libroop_rt.a $(CARGO_BIN)/libroop_rt.a
