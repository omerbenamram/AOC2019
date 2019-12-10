#!/usr/bin/env bash

cargo build --release;
/opt/perf record --call-graph dwarf ../target/release/day10 < input/astroid_map
/opt/perf script | inferno-collapse-perf > stacks.folded
cat stacks.folded | inferno-flamegraph > flamegraph.svg
