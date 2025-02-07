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
    @touch dev/SONE-061.avi
    @touch dev/SSIS-969.avi
    @touch dev/ACHJ-052.wmv
    @touch dev/SONE-388.wmv
    @cargo r

alias c := clean
# remove dev
clean:
    @rm -rf dev
