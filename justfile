_default:
    @just --list

alias t := try
# mkdir dev and run
try: clean
    @mkdir -p dev
    @touch dev/stars-804.mp4
    @touch dev/ipx-443.mp4
    @cargo r

alias c := clean
# remove dev
clean:
    @rm -rf dev
