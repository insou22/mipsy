#!/bin/bash
# Note, this presumes you have an ssh alias `cse` 

wasm-pack build --target no-modules --out-name wasm --out-dir ./dist --no-typescript
cd dist
ln -sf ../index.html
cd ..
NODE_ENV=production tailwindcss -c ./tailwind.config.js -o dist/tailwind.css --minify

if [ "$1" = "--push" ]; then

    scp dist/* cse:~/web/mipsy/

fi
