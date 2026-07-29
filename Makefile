.PHONY: all bundle build sign update-json clean dev

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

# Sign a build artifact for Tauri updater
# Usage: make sign FILE=path/to/appimage
sign:
	$(TAURI) signer sign \
		-k "$(TAURI_SIGNING_PRIVATE_KEY_PATH)" \
		-p "$(TAURI_SIGNING_PRIVATE_KEY_PASSWORD)" \
		"$(FILE)"

# Generate Tauri updater update.json (run after signed build)
update-json:
	@echo '{' > update.json
	@echo '  "version": "$(VERSION)",' >> update.json
	@echo '  "url": "$(URL)",' >> update.json
	@echo '  "signature": "$(SIG)",' >> update.json
	@echo '  "notes": "$(NOTES)"' >> update.json
	@echo '}' >> update.json
	@echo "update.json generated for version $(VERSION)"

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
