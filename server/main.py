"""
Frappe Desktop License & Update Server
=======================================
Run: uvicorn main:app --host 0.0.0.0 --port 8410

Endpoints:
  POST /v1/activate        - Activate a license key
  POST /v1/deactivate      - Deactivate a license key
  GET  /v1/updates/latest  - Check for updates
"""

from __future__ import annotations
from datetime import datetime, timedelta, timezone
from typing import Optional

from fastapi import FastAPI, HTTPException
from pydantic import BaseModel

app = FastAPI(title="Frappe Desktop API", version="1.0.0")

# ── In-memory store (replace with a database in production) ──
LICENSES: dict[str, dict] = {
    # "XXXX-XXXX-XXXX-XXXX": {"type": "perpetual", "max_activations": 3, "activations": []},
}

ACTIVATIONS: dict[str, list[str]] = {}  # license_key -> [machine_id, ...]

# ── Models ──────────────────────────────────────────────────

class ActivateRequest(BaseModel):
    machine_id: str
    license_key: str

class ActivateResponse(BaseModel):
    status: str
    expires_at: Optional[str] = None
    license_type: Optional[str] = None
    error: Optional[str] = None

class DeactivateRequest(BaseModel):
    machine_id: str
    license_key: str

class DeactivateResponse(BaseModel):
    status: str

class UpdateInfo(BaseModel):
    version: str
    url: str
    signature: str
    notes: str

# ── Endpoints ───────────────────────────────────────────────

@app.post("/v1/activate", response_model=ActivateResponse)
def activate(req: ActivateRequest):
    lic = LICENSES.get(req.license_key)
    if lic is None:
        return ActivateResponse(status="error", error="مفتاح الترخيص غير صالح")

    max_act = lic.get("max_activations", 1)
    acts = ACTIVATIONS.setdefault(req.license_key, [])

    if req.machine_id in acts:
        # Already activated on this machine — extend
        pass
    elif len(acts) >= max_act:
        return ActivateResponse(
            status="error",
            error=f"تم تجاوز الحد الأقصى من مرات التفعيل ({max_act})",
        )
    else:
        acts.append(req.machine_id)

    expires = None
    if lic["type"] == "subscription":
        expires = (datetime.now(timezone.utc) + timedelta(days=365)).isoformat()
    else:
        expires = (datetime.now(timezone.utc) + timedelta(days=3650)).isoformat()

    return ActivateResponse(
        status="ok",
        expires_at=expires,
        license_type=lic["type"],
    )


@app.post("/v1/deactivate", response_model=DeactivateResponse)
def deactivate(req: DeactivateRequest):
    acts = ACTIVATIONS.get(req.license_key, [])
    if req.machine_id in acts:
        acts.remove(req.machine_id)
    return DeactivateResponse(status="ok")


@app.get("/v1/updates/latest")
def latest_update():
    return UpdateInfo(
        version="1.0.1",
        url="https://releases.frappe-desktop.com/frappe-desktop_1.0.1_amd64.AppImage",
        signature="...",
        notes="إصلاحات أخطاء وتحسينات في الأداء",
    )


@app.get("/v1/health")
def health():
    return {"status": "ok"}


# ── Entry ───────────────────────────────────────────────────

if __name__ == "__main__":
    import uvicorn
    uvicorn.run("main:app", host="0.0.0.0", port=8410, reload=True)
