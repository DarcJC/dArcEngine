# dArCEngine

## Introduction

## Workspace Setup

### WASM

1. Install [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/).
2. Export `RUSTFLAGS` environment variable. `$env:RUSTFLAGS="--cfg=web_sys_unstable_apis"` (Optional in Windows/Powershell)
3. Build a wasm file `RUSTFLAGS=--cfg=web_sys_unstable_apis wasm-pack build renderer`
4. Start a http server to serve your files (i.e. `python -m http.server 30001`)
