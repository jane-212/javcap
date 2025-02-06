_default:
    @just --list

alias t := try
# mkdir dev and run
try: clean
    @mkdir -p dev
    @touch dev/stars-804.mp4
    @touch dev/ipx-443-1.mp4
    @touch dev/ipx-443-2.mp4
    @touch dev/fc2-1200809.mkv
    @cargo r

alias c := clean
# remove dev
clean:
    @rm -rf dev
