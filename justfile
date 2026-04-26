test *args:
    cargo test {{args}}

up:
    nix flake update
    cargo upgrade -i

fix: clippy-fix lint test

clippy-fix:
    cargo clippy --fix --allow-staged
    cargo fmt

check: lint test

lint: fmt-check clippy

fmt-check:
    cargo fmt --all -- --check

clippy:
    cargo clippy -- -D warnings
