all:
	cargo build --release
	mv ./target/release/sum-correction $(HOME)/.local/bin/.
