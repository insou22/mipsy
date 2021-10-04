tailwindcss -c ./tailwind.config.js -o ./static/tailwind.css
wasm-pack build --target no-modules --out-name wasm --out-dir ./static --no-typescript --dev
