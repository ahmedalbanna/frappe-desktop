# 🔐 Secrets Setup for Frappe Desktop

This document describes the required GitHub Secrets for building and deploying Frappe Desktop.

## Required Secrets

### Production Build

| Secret Name | Description | How to Generate |
|-------------|-------------|-----------------|
| `TAURI_SIGNING_PRIVATE_KEY` | RSA private key for signing updates | `npx tauri signer generate` |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Password for the signing key | Choose a strong password |

### Generating Keys

```bash
# Navigate to project root
cd frappe-desktop

# Generate signing keys
npx tauri signer generate --write-keys server/updater.key --password YOUR_SECURE_PASSWORD

# Output files:
# - server/updater.key (keep private!)
# - server/updater.key.pub (add to tauri.conf.json)
```

### Adding Secrets to GitHub

1. Go to: **Settings → Secrets and variables → Actions → New repository secret**
2. Add each secret:
   - `TAURI_SIGNING_PRIVATE_KEY` = contents of `server/updater.key`
   - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` = your password

### License Server (Optional)

For standalone license server functionality:
- Deploy `server/main.py` to your server
- Configure your domain in `tauri.conf.json` updater endpoint

## Testing Locally

To test the Windows build locally, install MinGW and NSIS:

```bash
# Ubuntu/Debian
sudo apt install wine wine64 mingw-w64 nsis

# Build for Windows
pnpm tauri build --targets nsis
```

## Security Notes

- **Never commit `server/updater.key` to version control!**
- Rotate signing keys periodically
- Use environment variables in production deployments