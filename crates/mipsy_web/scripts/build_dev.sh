wasm-pack build --target no-modules --out-name wasm --out-dir ./static --no-typescript --dev
cp index.html static/
cp dist/tailwind.css static/tailwind.css
