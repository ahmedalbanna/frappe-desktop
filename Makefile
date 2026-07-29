.PHONY: all bundle build clean

SHELL := /bin/bash
TAURI := pnpm tauri

all: bundle build

# Download all portable binaries
bundle:
	bash bundle/bundle-mariadb.sh
	bash bundle/bundle-redis.sh
	bash bundle/bundle-python.sh

# Build the Tauri application (debug)
build-debug:
	$(TAURI) build --debug --bundles none

# Build the Tauri application (release)
build:
	$(TAURI) build --bundles appimage

# Build only for AppImage
appimage:
	$(TAURI) build --bundles appimage

# Build only for NSIS (Windows cross-compile)
nsis:
	$(TAURI) build --bundles nsis

# Clean build artifacts
clean:
	rm -rf src-tauri/target
	rm -rf dist
	rm -rf src-tauri/resources/*

# Run in development mode
dev:
	$(TAURI) dev

# Run frontend in browser for quick UI dev
dev-ui:
	pnpm dev
