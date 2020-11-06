#!/bin/zsh
cd rspim_parser && cargo "$@" && cd .. && cargo "$@" && cd rspim && cargo "$@" && cd ..;
