#!/bin/bash

videos=( \
    "STARS-804.mp4" \
    "STARS804.mp4" \
    "SONE-143-1.mp4" \
    "SONE-143-2.mp4" \
    "FC2-PPV-1292936.mp4" \
    "https:www.javbus.com@Stars-804.mp4"
)
dev_dir="dev"
out="$dev_dir/out"

dev() {
    clear
    mkdir "$dev_dir"
    mkdir "$out"
    for video in ${videos[@]}
    do
        file="$out/$video"
        touch "$file"
    done
    cargo run
}

clear() {
    rm -rf "$dev_dir"
    rm -rf "logs"
}

help() {
    echo "dev - clear and run"
    echo "clear - clear all"
}

cmd=$1
if [ ! -z $cmd ]; then
    case $cmd in
        "dev")
            dev
            ;;
        "clear")
            clear
            ;;
        *)
            echo "command $cmd not found"
            help
    esac
else
    help
fi
