lint:
    cargo clippy --all-targets --all-features -- -D warnings
    cargo audit

run namespace:
    cargo run -- -n {{namespace}}

test:
    cargo test --all-targets --all-features
