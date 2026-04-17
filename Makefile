.PHONY: install restart

install:
	cargo install --path .

restart:
	-pkill -x rusty-panel
	rusty-panel &
