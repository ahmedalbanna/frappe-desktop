# Frappe Desktop — Turnkey ERP على سطح المكتب

## الرؤية
تحويل Frappe Framework من نظام يعتمد على CLI/خادم إلى **منتج ديسكتوب جاهز (Turnkey Desktop Solution)** يمكن لأي شخص تثبيته بنقرة واحدة دون الحاجة لمطوّر أو مسؤول أنظمة.

## المكونات
- **Shell**: Tauri v2 (Rust + WebView)
- **Frontend**: Vue 3 + TypeScript + Pinia
- **Backend**: Rust (Service Manager, Licensing, Backup)
- **DB**: MariaDB embedded (portable binary)
- **Cache**: Redis embedded (portable binary)
- **Engine**: Frappe v15 + Python embedded

## الأنظمة المستهدفة
- Windows 10/11 (x64) — NSIS installer
- Linux (x64) — AppImage / .deb
- macOS — قيد التطوير

## المعمارية

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
└──────────────────────┼───────────────────────────────┘
                       │
    ┌──────────────────┼──────────────────┐
    ▼                  ▼                  ▼
┌─────────┐     ┌──────────┐     ┌──────────────┐
│ MariaDB │     │  Redis   │     │  Frappe       │
│ (portable)    │ (portable)     │  (sidecar)    │
└─────────┘     └──────────┘     └──────────────┘
```

## الحجم التقريبي للمثبت

| المكون | الحجم |
|--------|-------|
| MariaDB portable | ~200 MB |
| Redis portable | ~15 MB |
| Python 3.12 embed | ~80 MB |
| Frappe + ERPNext | ~150 MB |
| Tauri executable | ~10 MB |
| **الإجمالي** | **~450 MB** (قابل للضغط ~200 MB) |

## الميزات
- ✅ First-Run Wizard (إعداد الموقع، المستخدم، الشركة)
- ✅ Dashboard (حالة الخدمات + Start/Stop)
- ✅ Backup & Restore (mysqldump + gzip)
- ✅ Licensing (Hardware ID + توليد/تحقق المفتاح)
- ✅ Auto-updater (Tauri updater plugin)
- ✅ Frappe Web App داخل WebView (أو متصفح خارجي)

## الأدوات المستخدمة

| الأداة | الإصدار |
|--------|---------|
| Rust | 1.97.1 |
| Tauri | v2 |
| Node.js | 20.20.2 |
| pnpm | 10.28.2 |
| Vue 3 | 3.5.x |
| Vite | 6.x |
| TypeScript | 5.7.x |

## البنية التجارية
- **نموذج الترخيص**: ترخيص دائم مربوط بالعتاد (Hardware-Locked) مع تفعيل أونلاين/أوفلاين
- **التسعير المقترح**:
  - ترخيص دائم: $299 (تحديثات سنة)
  - اشتراك سنوي: $99/year (تحديثات + دعم)
  - حزمة POS: $499 (مع جهاز POS مخصص)
- **التحديثات**: OTA عبر Tauri updater من سيرفر خاص
- **قنوات البيع**: موقع رسمي + متجر مايكروسوفت + شركاء

## هيكل المشروع

```
frappe-desktop/
├── src-tauri/               # Rust backend (Tauri)
│   ├── src/
│   │   ├── main.rs          # Entry point
│   │   ├── lib.rs           # Tauri commands
│   │   ├── runtime.rs       # Service manager
│   │   ├── backup.rs        # Backup/restore
│   │   └── licensing.rs     # License system
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/        # Tauri v2 permissions
│   ├── icons/               # App icons
│   └── resources/           # Bundled binaries
├── src/                     # Frontend (Vue 3)
│   ├── main.ts
│   ├── App.vue
│   ├── components/          # Reusable components
│   ├── views/               # Page views
│   └── assets/              # Static assets
├── bundle/                  # Bundle scripts
│   ├── bundle-mariadb.sh
│   ├── bundle-redis.sh
│   └── bundle-python.sh
├── package.json
├── vite.config.ts
└── idea.md                  # This file
```
