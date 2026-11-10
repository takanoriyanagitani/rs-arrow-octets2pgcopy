#!/bin/sh

set -u

wsm="./arrow-array-log2pgcopy.wasm"

run_wasmtime() {
	wasmtime \
		run \
		"${wsm}"
}

run_wasi() {
	run_wasmtime
}

run_wasmtime | node example.mjs
