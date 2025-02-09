_default:
    @just --list

alias t := try
# mkdir dev and run
try: clean
    @mkdir -p dev
    @touch dev/FC2-PPV-3061625.wmv
    @touch dev/HEYZO-3525.wmv
    @touch dev/MD-0260.wmv
    @touch dev/PROB-3.wmv
    @touch dev/ROYD-108.wmv
    @touch dev/stars-804-1.wmv
    @touch dev/stars-804-2.wmv
    @touch dev/ipx-443.wmv
    @touch dev/ipx-443-1.wmv
    @cargo r

alias c := clean
# remove dev
clean:
    @rm -rf dev
