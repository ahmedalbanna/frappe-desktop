# Frappe Desktop

A turnkey desktop solution for Frappe Framework, providing a single-click installer and controller for Frappe/ERPNext without requiring CLI or developer setup.

## Features

- **One-Click Installer**: Single `.exe` (Windows), `.dmg` (macOS), or `.AppImage` (Linux) installer
- **No-Code Admin UI**: Simple graphical interface in Arabic and English
- **Service Management**: Start/stop/backup MariaDB, Redis, and Frappe services
- **Backup & Restore**: Take backups of database and files with gzip compression
- **Licensing**: Hardware-locked license system with online/offline activation options
- **Auto-Updates**: Automatic updates via Tauri updater plugin

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                    Tauri Shell                       │
│  ┌───────────────────────────────────────────────┐  │
│  │         Vue 3 + Frappe UI Frontend            │  │
│  │  (Dashboard, Settings, Backups, License)      │  │
│  └───────────────────┬───────────────────────────┘  │
│                      │ IPC (invoke)                  │
│  ┌───────────────────▼───────────────────────────┐  │
│  │             Rust Backend (Core)                │  │
│  │                                                │  │
│  │  ┌──────────────┐   ┌──────────────────┐      │  │
│  │  │ Service Mgr   │   │  License Mgr     │      │  │
│  │  │ - MariaDB     │   │  - Hardware ID   │      │  │
│  │  │ - Redis       │   │  - Offline/Online│      │  │
│  │  │ - Frappe      │   │  - Activation    │      │  │
│  │  └──────────────┘   └──────────────────┘      │  │
│  │  ┌──────────────┐   ┌──────────────────┐      │  │
│  │  │ Backup Mgr    │   │  Updater         │      │  │
│  │  │ - mysqldump   │   │  - AppImage/.exe │      │  │
│  │  │ - site files  │   │  - Auto-update   │      │  │
│  │  └──────────────┘   └──────────────────┘      │  │
│  └───────────────────┬───────────────────────────┘  │
└───────────────────────┼───────────────────────────────┘
                        │
     ┌──────────────────┼──────────────────┐
     ▼                  ▼                  ▼
 ┌─────────┐     ┌──────────┐     ┌──────────────┐
 │ MariaDB │     │  Redis   │     │  Frappe       │
 │ (portable) │   │ (portable)   │  (sidecar)    │
 └─────────┘     └──────────┘     └──────────────┘
```

## Requirements

- **OS**: Windows 10/11, Linux (x64), macOS (beta)
- **Node.js**: 20.x
- **pnpm**: 10.x
- **Rust**: 1.97.1+
- **Tauri**: v2

## Building

```bash
# Install dependencies
pnpm install

# Download bundled binaries (MariaDB, Redis, Python)
bash bundle/bundle-mariadb.sh
bash bundle/bundle-redis.sh
bash bundle/bundle-python.sh

# Development build
pnpm tauri dev

# Production build
pnpm tauri build

# Build for specific targets
pnpm tauri build --bundles appimage  # Linux
pnpm tauri build --bundles nsis      # Windows
```

## Project Structure

```
frappe-desktop/
├── src/                     # Vue 3 Frontend
│   ├── App.vue
│   ├── main.ts
│   ├── components/          # Reusable UI components
│   └── views/               # Page views
├── src-tauri/               # Rust Backend
│   ├── src/
│   │   ├── lib.rs          # Main entry point
│   │   ├── runtime.rs      # Service management
│   │   ├── licensing.rs    # License system
│   │   ├── backup.rs       # Backup/restore
│   │   └── setup.rs        # Site setup
│   ├── Cargo.toml
│   └── tauri.conf.json
├── bundle/                  # Bundle scripts
│   ├── bundle-mariadb.sh
│   ├── bundle-redis.sh
│   └── bundle-python.sh
└── idea.md                  # Project vision
```

## License

This project is licensed under the MIT License. See [LICENSE.txt](LICENSE.txt) for details.

## Contributing

Contributions are welcome! Please feel free to submit pull requests.