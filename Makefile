SRC=src

all: ropc

ropc: $(SRC)/* Cargo.toml
	cargo build; \
  cp target/debug/ropc .

release: $(SRC)/* Cargo.toml
	cargo build --release; \
  cp target/debug/ropc .

clean:
	rm Cargo.lock ropc
	rm -rf target
