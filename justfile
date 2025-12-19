build-web:
    cargo build --release --target wasm32-unknown-unknown
    wasm-bindgen \
        --no-typescript \
        --target web \
        --out-dir ./web/wasm/ \
        --out-name "holiday_card" \
        ./target/wasm32-unknown-unknown/release/holiday_card.wasm
    wasm-opt \
        -Oz \
        -o ./web/wasm/holiday_card_bg.wasm \
        ./web/wasm/holiday_card_bg.wasm

run-web: build-web
    python3 -m http.server 8888

stats:
    tokei .
