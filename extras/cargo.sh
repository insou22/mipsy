#!/bin/zsh
cd mipsy_parser && cargo "$@" && cd .. && cargo "$@" && cd mipsy && cargo "$@" && cd ..;
