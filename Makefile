all: help

.PHONY: help
help:
	@echo "run(r) - Run the app"
	@echo "clean - Clean caches"
	@echo "reset - Reset dev directory"
	@echo "test(t) - Run the tests"

.PHONY: run r
r: run
run: reset
	@zig build run

.PHONY: clean
clean:
	@rm -rf input
	@rm -rf output
	@rm -rf zig-pkg
	@rm -rf zig-out
	@rm -rf .zig-cache

.PHONY: reset
reset:
	@rm -rf input
	@rm -rf output
	@mkdir input
	@touch input/stars-804.mp4
	@mkdir input/ok
	@touch input/ok/ipx-144.mkv

.PHONY: test t
t: test
test:
	@zig build test
