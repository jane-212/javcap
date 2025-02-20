_default:
    @just --list

alias t := try
# mkdir dev and run
try: remove
    @mkdir -p dev
    @touch dev/FC2-PPV-3061625.wmv
    @touch dev/FC2-PPV-1292936.wmv
    @touch dev/HEYZO-3525.wmv
    @touch dev/ROYD-108.wmv
    @touch dev/stars-804.wmv
    @touch dev/ipx-443.wmv
    @touch dev/FC2-PPV-4554988.wmv
    @touch dev/PRED-323.wmv
    @touch dev/md-0331.wmv
    @touch dev/小飞棍来咯.wmv
    @touch dev/GOIN-002.wmv
    @touch dev/GOIN-003.wmv
    @cargo r

log_file := home_directory() / ".cache" / "javcap" / "log"
alias l := log
# print local log
log:
    @cat {{log_file}}

config_file := home_directory() / ".config" / "javcap" / "config.toml"
editor := env("EDITOR", "vim")
alias c := config
# edit local config file
config:
    @{{editor}} {{config_file}}

alias r := remove
# remove dev
remove:
    @rm -rf dev

alias f := format
# format code
format:
    @cargo fmt
