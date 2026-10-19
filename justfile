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

# Build web/dist for hosting under a base path, as the Pages workflow does.
site base="/clattertrap/":
    cargo xtask site {{base}}

# Rewrite the README leaderboard from gallery/.
readme:
    cargo xtask readme

# Re-simulate every gallery entry and compare stored results.
verify:
    cargo run --package clattertrap-cli -- verify gallery

# Record the arena grid for one level and arm to docs/hero.gif and docs/hero.mp4.
hero level="seesaw-launch" arm="blueprint":
    cargo xtask hero {{level}} {{arm}}

# Contestant runs against clattertrap mcp, on every level or one: Claude Code headless, or a chat endpoint.
pilot model runs arm level="" endpoint="":
    cargo run --release --package clattertrap-cli -- arena --model {{model}} --runs {{runs}} --arm {{arm}} {{ if level == "" { "" } else { "--level " + level } }} {{ if endpoint == "" { "" } else { "--endpoint " + endpoint } }}

# Criterion benches, including run_level.
bench:
    cargo bench -p clattertrap-core

# Fuzz a parser with random and mutated input for a time budget in seconds. Targets: scene.
fuzz target secs="60":
    cargo xtask fuzz {{target}} {{secs}}
