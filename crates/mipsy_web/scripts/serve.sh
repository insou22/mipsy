#!/bin/bash
# I typically run this as a background process and pipe to dev null
# ie scripts/serve &>/dev/null &
if ! command -v python3 2>&1 >/dev/null;
then
	echo 'error: you must install python3'
	exit
fi

cd static/
python3 -m http.server 3000
