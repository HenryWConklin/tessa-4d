#!/bin/bash
set -e

cargo build --package tessa4d-gdext
pushd itest/tessa4d-gdext
# Launch editor to make sure the .godot dir is set up, 
# e.g. with class_name scripts added to global scope.
godot --editor --quit
godot --script test_entrypoint.gd --fixed-fps 60 --windowed --resolution 1280x720 -- $@
popd
