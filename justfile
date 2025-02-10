_default:
    @just --list

alias t := try
# mkdir dev and run
try: remove
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
