CARGO = cargo
BINARY_NAME = fdrd
TARGET = arm-unknown-linux-musleabihf
BINARY := target/$(TARGET)/release/$(BINARY_NAME)

.PHONY: all
all: $(BINARY)

$(BINARY): $(wildcard src/*) Cargo.toml Cargo.lock flake.nix flake.lock
	nix develop .#pizero -c -- $(CARGO) build --target arm-unknown-linux-musleabihf --release

.PHONY: clean
clean:
	$(CARGO) clean
