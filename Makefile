.PHONY: run clean

all:
	@echo "run - clean and run"
	@echo "clean - just clean"
	
run: clean
	@mkdir -p dev
	@touch dev/STARS-997.mp4
	@touch dev/STARS997.mp4
	@touch dev/SONE-143-1.mp4
	@touch dev/SONE-143-2.mp4
	@touch dev/FC2-PPV-1292936.mp4
	@mkdir -p dev/stars
	@touch dev/stars/stars-804.mp4
	@cargo r

clean:
	@rm -rf dev
	@rm -rf logs