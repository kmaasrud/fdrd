PIZERO_IP = 192.168.0.60

CARGO = cargo
BINARY_NAME = fdrd
TARGET = arm-unknown-linux-musleabihf
BINARY := target/$(TARGET)/release/$(BINARY_NAME)

.PHONY: all
all: $(BINARY)

.PHONY: deploy
deploy: $(BINARY)
	scp $(BINARY) "$(PIZERO_IP):/home/$$USER/"
	ssh $(PIZERO_IP) 'sudo systemctl stop $(BINARY_NAME) && \
					  sudo mv fdrd /usr/local/bin/ && \
					  sudo systemctl start $(BINARY_NAME)'

$(BINARY): $(wildcard src/*) Cargo.toml Cargo.lock flake.nix flake.lock
	nix develop .#pizero -c -- $(CARGO) build --target arm-unknown-linux-musleabihf --release

.PHONY: clean
clean:
	$(CARGO) clean
