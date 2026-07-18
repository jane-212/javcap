all: help

.PHONY: help
help:
	@echo "run(r) - Run the app"

.PHONY: run r
r: run
run:
	zig build run
