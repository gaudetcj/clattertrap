set windows-shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-Command"]

# The merge gate: fmt, clippy (native and wasm32), prose lint, nextest, doc tests, wasm tests, web lint and tests, gallery verify.
ci:
    cargo xtask ci

# Run the Rust test suite.
test:
    cargo nextest run --workspace

# Build clattertrap-wasm into web/pkg/ with the pinned wasm-bindgen.
wasm:
    cargo xtask wasm

# Core tests on wasm32-wasip1 under the pinned wasmtime, plus the wrapper tests on wasm32-unknown-unknown.
wasm-test:
    cargo xtask wasm-test

# Viewer dev server.
web:
    npm --prefix web run dev

# Re-simulate every gallery entry and compare stored results (milestone 6).
verify:
    cargo xtask stub verify

# Record the arena grid to docs/hero.gif and docs/hero.mp4 (milestone 6).
hero:
    cargo xtask stub hero

# Headless Claude Code contestant runs against clattertrap mcp (milestone 5).
pilot model runs arm:
    cargo xtask stub pilot

# Criterion benches, including run_level.
bench:
    cargo bench -p clattertrap-core

# Fuzz a parser with random and mutated input for a time budget in seconds. Targets: scene.
fuzz target secs="60":
    cargo xtask fuzz {{target}} {{secs}}
