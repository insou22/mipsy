#!/usr/bin/env bash
if ! command -v wasm-pack 2>&1 >/dev/null;
then
	echo 'error: you must install wasm-pack'
	exit
fi

wasm-pack build --target no-modules --out-name wasm --out-dir ./static --no-typescript --dev
cp _static/index.html static/
cp -r _static/package static/
cp _static/tailwind.css static/

#if [ ! -f "static/tailwind.css" ];
#then
#	tailwindcss -o ./static/tailwind.css
#elif [ -f "dist/tailwind.css" ]; then
#	cp dist/tailwind.css static/tailwind.css
#else 
#  echo "no tailwind file, running"
#  tailwindcss -o ./static/tailwind.css
#fi

