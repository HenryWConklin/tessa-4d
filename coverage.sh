#!/bin/bash

# FIXME This breaks if there are multiple crates in the repo, only pulls coverage for the one named "tessa4d".
TEST_BIN=$(RUSTFLAGS="-C instrument-coverage" cargo test 2>&1 | tee coverage-test-out.txt | egrep -o 'target/debug/deps/tessa4d-[a-z0-9]+')
echo "Test binary: $TEST_BIN"
llvm-profdata merge -sparse */default_*.profraw default_*.profraw -o tessa4d.profdata
llvm-cov show --instr-profile=tessa4d.profdata --object "$TEST_BIN" --ignore-filename-regex="/.cargo/registry" --ignore-filename-regex="rustc"> coverage-report.txt
llvm-cov report --instr-profile=tessa4d.profdata --object "$TEST_BIN" --ignore-filename-regex="/.cargo/registry" --ignore-filename-regex="rustc" 
rm */default_*.profraw default_*.profraw