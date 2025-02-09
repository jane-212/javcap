_default:
    @just --list

alias t := try
# mkdir dev and run
try: clean
    @mkdir -p dev
    @touch dev/FC2-PPV-3061625.wmv
    @touch dev/MD-0260.wmv
    @touch dev/FC2-PPV-3572974.wmv
    @touch dev/PROB-3.wmv
    @touch dev/FC2-PPV-1932561.wmv
    @touch dev/ROYD-108.wmv
    @touch dev/FC2-PPV-1425988.wmv
    @cargo r

alias c := clean
# remove dev
clean:
    @rm -rf dev
