build:
	cargo build --release

test:
	cargo test

install:
	cargo install --path .
	mkdir -p ${HOME}/.config/boli/modules
	rm -rf ${HOME}/.config/boli/modules/*
	cp -r modules/* ${HOME}/.config/boli/modules