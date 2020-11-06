#!/bin/zsh
cd rspim_parser && cargo b "$@" && cd .. && cargo b "$@" && cd rspim && cargo b "$@" && cd ..;
