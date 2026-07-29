<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import FirstRunWizard from "./views/FirstRunWizard.vue";
import Dashboard from "./views/Dashboard.vue";
import Settings from "./views/Settings.vue";

type View = "loading" | "license" | "wizard" | "dashboard" | "settings";

interface LicenseInfo {
  key: string;
  machine_id: string;
  license_type: string;
  activated_at: string;
  expires_at: string;
  valid: boolean;
}

const currentView = ref<View>("loading");
const licenseInfo = ref<LicenseInfo | null>(null);

async function checkLicense() {
  try {
    const lic: LicenseInfo = await invoke("get_license_status");
    licenseInfo.value = lic;
    if (lic.valid) {
      await checkSetup();
    } else {
      currentView.value = "license";
    }
  } catch {
    currentView.value = "license";
  }
}

async function checkSetup() {
  try {
    const status = await invoke<{ configured: boolean }>("check_setup_status");
    currentView.value = status.configured ? "dashboard" : "wizard";
  } catch {
    currentView.value = "wizard";
  }
}

function goTo(view: View) {
  currentView.value = view;
}

function onLicensed() {
  checkSetup();
}

const licenseKey = ref("");
const offlineSig = ref("");
const activationMode = ref<"online" | "offline">("online");
const activating = ref(false);
const activationError = ref("");

async function activateOnline() {
  activating.value = true;
  activationError.value = "";
  try {
    licenseInfo.value = await invoke("activate_license_online", { key: licenseKey.value });
    onLicensed();
  } catch (e: any) {
    activationError.value = typeof e === "string" ? e : e.message || "فشل التفعيل";
  } finally {
    activating.value = false;
  }
}

async function activateOffline() {
  activating.value = true;
  activationError.value = "";
  try {
    licenseInfo.value = await invoke("activate_license_offline", { key: licenseKey.value, signature: offlineSig.value });
    onLicensed();
  } catch (e: any) {
    activationError.value = typeof e === "string" ? e : e.message || "فشل التفعيل";
  } finally {
    activating.value = false;
  }
}

async function startTrial() {
  activating.value = true;
  activationError.value = "";
  try {
    licenseInfo.value = await invoke("start_trial");
    onLicensed();
  } catch (e: any) {
    activationError.value = typeof e === "string" ? e : e.message || "فشل التفعيل التجريبي";
  } finally {
    activating.value = false;
  }
}

onMounted(checkLicense);
</script>

<template>
  <div class="app-shell">
    <!-- Loading -->
    <div v-if="currentView === 'loading'" class="loading-screen">
      <div class="spinner"></div>
      <p>جاري التحميل...</p>
    </div>

    <!-- License Gate -->
    <div v-else-if="currentView === 'license'" class="license-gate">
      <div class="license-card">
        <div class="gate-logo">FD</div>
        <h1>Frappe Desktop</h1>
        <p class="gate-subtitle">يرجى تفعيل الترخيص للبدء</p>

        <div class="mode-tabs">
          <button :class="['tab', { active: activationMode === 'online' }]" @click="activationMode = 'online'">تفعيل أونلاين</button>
          <button :class="['tab', { active: activationMode === 'offline' }]" @click="activationMode = 'offline'">تفعيل يدوي</button>
          <button class="tab trial-tab" @click="startTrial" :disabled="activating">نسخة تجريبية</button>
        </div>

        <div v-if="activationMode === 'online'" class="form-group">
          <label>مفتاح الترخيص</label>
          <input v-model="licenseKey" type="text" placeholder="أدخل مفتاح الترخيص" :disabled="activating" />
          <button class="btn btn-primary" @click="activateOnline" :disabled="activating || !licenseKey">
            {{ activating ? 'جاري التفعيل...' : 'تفعيل' }}
          </button>
        </div>

        <div v-if="activationMode === 'offline'" class="form-group">
          <label>مفتاح الترخيص</label>
          <input v-model="licenseKey" type="text" placeholder="أدخل مفتاح الترخيص" :disabled="activating" />
          <label>رمز التفعيل</label>
          <input v-model="offlineSig" type="text" placeholder="أدخل رمز التفعيل" :disabled="activating" />
          <button class="btn btn-primary" @click="activateOffline" :disabled="activating || !licenseKey || !offlineSig">
            {{ activating ? 'جاري التفعيل...' : 'تفعيل' }}
          </button>
        </div>

        <p v-if="activationError" class="msg-error">{{ activationError }}</p>
      </div>
    </div>

    <!-- Main App -->
    <div v-else>
      <nav class="top-nav">
        <div class="nav-brand">
          <span class="nav-logo">FD</span>
          <span class="nav-title">Frappe Desktop</span>
        </div>
        <div class="nav-links">
          <button
            :class="['nav-link', { active: currentView === 'dashboard' }]"
            @click="goTo('dashboard')"
          >لوحة التحكم</button>
          <button
            :class="['nav-link', { active: currentView === 'settings' }]"
            @click="goTo('settings')"
          >الإعدادات</button>
        </div>
      </nav>
      <FirstRunWizard v-if="currentView === 'wizard'" @complete="goTo('dashboard')" />
      <Dashboard v-if="currentView === 'dashboard'" />
      <Settings v-if="currentView === 'settings'" />
    </div>
  </div>
</template>

<style>
* { margin: 0; padding: 0; box-sizing: border-box; }
body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  background: #f0f2f5; color: #1a1a2e; direction: rtl;
  -webkit-font-smoothing: antialiased;
}
.app-shell { min-height: 100vh; }

.loading-screen {
  display: flex; flex-direction: column; align-items: center;
  justify-content: center; height: 100vh; gap: 1rem;
}
.spinner {
  width: 40px; height: 40px; border: 3px solid #e5e7eb;
  border-top-color: #2563eb; border-radius: 50%;
  animation: spin 0.8s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }

.license-gate {
  display: flex; align-items: center; justify-content: center;
  min-height: 100vh; padding: 2rem;
}
.license-card {
  background: white; border-radius: 16px; padding: 2.5rem;
  box-shadow: 0 4px 24px rgba(0,0,0,0.1); max-width: 480px; width: 100%;
  text-align: center;
}
.gate-logo {
  background: #2563eb; color: white; width: 64px; height: 64px;
  display: flex; align-items: center; justify-content: center;
  border-radius: 16px; font-weight: 800; font-size: 1.5rem; margin: 0 auto 1rem;
}
.license-card h1 { font-size: 1.5rem; color: #1f2937; margin-bottom: 0.5rem; }
.gate-subtitle { color: #6b7280; margin-bottom: 1.5rem; font-size: 0.9rem; }

.mode-tabs { display: flex; gap: 0.5rem; margin-bottom: 1.5rem; justify-content: center; }
.tab { padding: 0.4rem 1rem; border: 1px solid #e5e7eb; border-radius: 6px; background: #f9fafb; cursor: pointer; font-size: 0.8rem; font-weight: 600; }
.tab.active { background: #2563eb; color: white; border-color: #2563eb; }
.trial-tab { background: #fef3c7; border-color: #f59e0b; color: #92400e; }
.trial-tab:disabled { opacity: 0.5; cursor: not-allowed; }

.form-group { display: flex; flex-direction: column; gap: 0.5rem; margin-bottom: 0.75rem; text-align: right; }
.form-group label { font-size: 0.8rem; color: #6b7280; font-weight: 600; }
.form-group input { padding: 0.5rem; border: 1px solid #d1d5db; border-radius: 6px; font-size: 0.9rem; direction: ltr; text-align: left; }
.form-group input:focus { outline: none; border-color: #2563eb; box-shadow: 0 0 0 3px rgba(37,99,235,0.1); }
.btn { padding: 0.5rem 1rem; border: none; border-radius: 6px; font-size: 0.85rem; cursor: pointer; font-weight: 600; }
.btn:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-primary { background: #2563eb; color: white; }

.msg-error { color: #dc2626; font-size: 0.85rem; margin-top: 0.75rem; }

.top-nav {
  display: flex; justify-content: space-between; align-items: center;
  padding: 0.75rem 1.5rem; background: white;
  border-bottom: 1px solid #e5e7eb; box-shadow: 0 1px 2px rgba(0,0,0,0.05);
}
.nav-brand { display: flex; align-items: center; gap: 0.5rem; }
.nav-logo {
  background: #2563eb; color: white; width: 32px; height: 32px;
  display: flex; align-items: center; justify-content: center;
  border-radius: 8px; font-weight: 800; font-size: 0.85rem;
}
.nav-title { font-weight: 700; color: #1f2937; }
.nav-links { display: flex; gap: 0.25rem; }
.nav-link {
  padding: 0.4rem 1rem; border: none; border-radius: 6px;
  background: transparent; color: #6b7280; font-size: 0.85rem;
  font-weight: 600; cursor: pointer;
}
.nav-link.active { background: #eff6ff; color: #2563eb; }
.nav-link:hover { background: #f3f4f6; }
</style>
