.PHONY: all clean release deb test build

all: test build

build:
	cargo build --release

release:
	cargo build --release

test:
	cargo test

clean:
	cargo clean
	rm -rf debian-binary
	rm -f *.deb

deb: release
	dpkg-buildpackage -us -uc -b

deb-local: release
	dpkg-buildpackage -us -uc -b -nc

# Install the deb package
install-deb:
	sudo dpkg -i ../nanoimage_0.1.0_amd64.deb

# Build and install in one step
build-install: deb install-deb

# Clean and rebuild
rebuild: clean deb
