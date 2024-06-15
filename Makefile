.PHONY: build build-tessa build-bevy \
	check check-tessa check-bevy \
	itest itest-bevy

build: build-tessa build-bevy

build-tessa:
	cargo build --package tessa4d 

build-bevy: 
	cargo build --package tessa4d-bevy

clean:
	cargo clean

check: check-tessa check-bevy

check-tessa:
	cargo clippy --package tessa4d
	cargo fmt --check --package tessa4d

check-bevy:
	cargo clippy --package tessa4d-bevy
	cargo fmt --check --package tessa4d-bevy

itest: itest-bevy

itest-bevy:
	cargo test --package tessa4d-bevy
