#!/bin/sh

cargo \
	watch \
	--shell ./check.sh \
	--watch ./src \
	--watch ./examples \
	--watch ./Cargo.toml
