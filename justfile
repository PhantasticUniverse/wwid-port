# WIDesigner Port — Build Tasks

# Run all Rust tests
test:
    cd wid && cargo test

# Build WASM module + wasm-bindgen
wasm:
    cd wid && cargo build --target wasm32-unknown-unknown --release -p wid-wasm
    cd wid && wasm-bindgen target/wasm32-unknown-unknown/release/wid_wasm.wasm --out-dir ../web/wasm --target web

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

# Re-import reference articles + Wood Wind tuning from local-flute-encyclopedia
import-reference:
    node tools/import-reference.mjs

# Build web with a GitHub Pages subpath base (local deploy verification)
pages BASE="/wwid-port/": wasm
    cd web && npm install && npx vite build --base={{BASE}}
