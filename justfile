# WIDesigner Port — Build Tasks

# Run all Rust tests
test:
    cd wid && cargo test

# Build WASM module + wasm-bindgen
wasm:
    cd wid && cargo build --target wasm32-unknown-unknown --release -p wid-wasm
    cd wid && wasm-bindgen target/wasm32-unknown-unknown/release/wid_wasm.wasm --out-dir crates/wid-wasm/pkg --target web

# Build web frontend
web:
    cd web && npm install && npx vite build

# Full build pipeline: test + wasm + web
build: test wasm web

# WASM + dev server
dev: wasm
    cd web && npm install && npx vite

# Fetch oracle + build golden harness
fixtures:
    ./tools/fetch-oracle.sh
    cd golden-harness && ./gradlew build
