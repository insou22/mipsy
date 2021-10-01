A browser interface for debugging MIPS programs with mipsy as a backend. 

## Run
1) `./build_dev.sh` will compile the relevant rust code to wasm files, and place into the static directory
2) `./serve.sh` will simply run a http client inside `static/` to serve the relevant files. 

## Debug
If you have no CSS - you will need to produce `tailwind.css` file , `./purge_tailwind.sh` should help here. 

tailwind setup with https://dev.to/arctic_hen7/how-to-set-up-tailwind-css-with-yew-and-trunk-il9

