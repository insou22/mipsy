#!/usr/bin/env bash
if ! command -v wasm-pack 2>&1 >/dev/null;
then
	echo 'error: you must install wasm-pack'
	exit
fi

wasm-pack build --target no-modules --out-name wasm --out-dir ./static --no-typescript --dev
cp index.html static/

if [ ! -f "dist/tailwind.css" -o ! -f "static/tailwindcss" ];
then
	tailwindcss -c ./tailwind.config.js -o ./static/tailwind.css
else
	cp dist/tailwind.css ./static/tailwind.css
fi

cp dist/tailwind.css static/tailwind.css
