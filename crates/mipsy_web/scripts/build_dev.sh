#!/usr/bin/env bash
if ! command -v wasm-pack 2>&1 >/dev/null;
then
	echo 'error: you must install wasm-pack'
	exit
fi

wasm-pack build --target no-modules --out-name wasm --out-dir ./static --no-typescript --dev
cp _static/index.html static/
cp -r _static/package static/


if [ ! -f "dist/tailwind.css" -a ! -f "static/tailwind.css" ];
then
	tailwindcss -o ./static/tailwind.css
else
	cp dist/tailwind.css static/tailwind.css
fi

