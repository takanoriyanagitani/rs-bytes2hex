#!/bin/bash

export RUSTFLAGS='-C target-feature=+simd128,+relaxed-simd'
cargo \
	build \
	--target wasm32-wasip1 \
	--profile release-wasi
