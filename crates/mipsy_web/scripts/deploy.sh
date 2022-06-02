#!/usr/bin/env bash 
# This script will build and deploy to UNSW CSE servers
# presumes you have an ssh alias 'cse'


if ! command -v trunk 2>&1 >/dev/null;
then
	echo 'error: you must install trunk (try the instructions at https://trunkrs.dev/)'
	exit
fi

if ! command -v tailwindcss 2>&1 >/dev/null;
then
	echo 'error: you must install tailwindcss (try `npm i -g tailwindcss`)'
	exit
fi

CSE_ACCOUNT=
PUBLIC_URL="/"
while true; do
    case "$1" in 
        --cse_account=*)
            val=${1#*=}
            CSE_ACCOUNT="$val"; shift; 
            ;;
        --public_url=*)             
            val=${1#*=}
            PUBLIC_URL="$val"; shift; 
            ;;
        * ) break ;;
    esac
done

trunk build --release --public-url=$PUBLIC_URL

if [ "$CSE_ACCOUNT" = "both" ]; then
    scp -r dist/* cse:~cs1521/web/mipsy
    scp -r dist/* cse:~/web/mipsy
else
    scp -r dist/* cse:~$CSE_ACCOUNT/web/mipsy/
fi
