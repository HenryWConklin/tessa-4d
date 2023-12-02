.PHONY: build build-tessa build-gdext build-bevy \
	check check-tessa check-gdext check-bevy \
	itest itest-gdext itest-bevy

GDEXT_LIBS=target/debug/libtessa4d_gdext.so

build: build-tessa build-gdext build-bevy

build-tessa:
	cargo build --package tessa4d 

build-gdext: target/debug/libtessa4d_gdext.so 

build-bevy: 
	cargo build --package tessa4d-bevy

clean:
	cargo clean

check: check-tessa check-gdext check-bevy

check-tessa:
	cargo clippy --package tessa4d
	cargo fmt --check --package tessa4d

check-gdext:
	cargo clippy --package tessa4d-gdext
	cargo fmt --check --package tessa4d-gdext

check-bevy:
	cargo clippy --package tessa4d-bevy
	cargo fmt --check --package tessa4d-bevy

itest: itest-gdext itest-bevy

itest-gdext: $(GDEXT_LIBS)
# Open in the editor first to update the .godot dir
	godot --editor --quit --path itest/tessa4d-gdext
	godot --script test_entrypoint.gd --fixed-fps 60 --windowed --resolution 1280x720 --path itest/tessa4d-gdext

itest-gdext-record-screenshots: $(GDEXT_LIBS)
# Open in the editor first to update the .godot dir
	godot --editor --quit --path itest/tessa4d-gdext
	godot --script test_entrypoint.gd --fixed-fps 60 --windowed --resolution 1280x720 --path itest/tessa4d-gdext -- --record-screenshots

itest-bevy:
	cargo test --package tessa4d-bevy

### Actual files

target/debug/libtessa4d_gdext.so: Cargo.toml Cargo.lock $(shell find tessa4d-gdext tessa4d -type f)
	cargo build --package tessa4d-gdext
	touch target/debug/libtessa4d_gdext.so