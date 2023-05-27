# dArCEngine

[![Bench](https://github.com/DarcJC/dArcEngine/actions/workflows/rust.yml/badge.svg)](https://github.com/DarcJC/dArcEngine/actions/workflows/rust.yml)

## Introduction

## Workspace Setup

### Windows

1. Export `$env:CARGO_BUILD_TARGET="x86_64-pc-windows-msvc"`

### WASM

1. `cargo install wasm-server-runner`
2. `rustup target add wasm32-unknown-unknown`
3. Export `RUSTFLAGS` environment variable. `$env:RUSTFLAGS="--cfg=web_sys_unstable_apis"`
4. `cargo run --target wasm32-unknown-unknown`

Ignore below for now.

1. Install [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/).
2. Install wasm-bindgen-cli (`cargo install wasm-bindgen-cli`).
3. Export `RUSTFLAGS` environment variable. `$env:RUSTFLAGS="--cfg=web_sys_unstable_apis"` (Optional in Windows/Powershell)
4. Build a wasm file `RUSTFLAGS=--cfg=web_sys_unstable_apis wasm-pack build renderer --target web --features webgl`
5. Start a http server to serve your files (i.e. `python -m http.server 30001`)
