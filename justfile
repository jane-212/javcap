_default:
    @just --list

alias t := try
# mkdir dev and run
try: remove
    @mkdir -p dev
    @touch dev/FC2-PPV-3061625.wmv
    @touch dev/cawd-773.wmv
    @touch dev/小飞棍来咯.wmv
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
