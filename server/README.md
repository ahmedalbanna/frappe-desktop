# Frappe Desktop License Server 

Server API for managing licenses and updates.

## Run Locally

```bash
pip install -r requirements.txt
uvicorn main:app --port 8410 --reload
```

## API Endpoints

### POST /v1/activate

Activate a license key.

Request:
```json
{
  "machine_id": "string",
  "license_key": "XXXX-XXXX-XXXX-XXXX"
}
```

Response:
```json
{
  "status": "ok" | "error",
  "expires_at": "2025-01-01T00:00:00Z",
  "license_type": "perpetual" | "subscription",
  "error": "description"
}
```

### POST /v1/deactivate

Deactivate a license key.

Request:
```json
{
  "machine_id": "string",
  "license_key": "string"
}
```

Response:
```json
{
  "status": "ok"
}
```

### GET /v1/updates/latest

Check for latest update.

Response:
```json
{
  "version": "1.0.0",
  "url": "https://releases.frappe-desktop.com/frappe-desktop_1.0.0_amd64.AppImage",
  "signature": "base64-signature",
  "notes": "Changelog notes"
}
```

## Deployment

For production, deploy to a HTTPS-enabled server and update `tauri.conf.json`:

```json
"updater": {
  "endpoints": ["https://YOUR_SERVER/v1/updates/latest"]
}
```

## Tauri Signing

The updater requires signing with RSA keys. Generate:

```bash
npx tauri signer generate --write-keys updater.key --password YOUR_PASSWORD
```

Add the public key to `tauri.conf.json` updater `pubkey` field.