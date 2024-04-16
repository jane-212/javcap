#!/bin/bash

run()
{
    clear
    out="videos"
    mkdir $out
    lines=("STARS-804.mp4" \
    "SONE-143.mp4" \
    "FC2-PPV-1292936.mp4" \
    "ABF-047.mp4")
    for line in ${lines[@]}
    do
        touch $out/$line
    done
    cargo run
}

clear() {
    rm -rf output
    rm -rf logs
    rm -rf videos
    rm -rf other
}

case $1 in
    "run")
        run
    ;;
    "clear")
        clear
    ;;
    *)
        echo "command $1 not allowed"
esac
