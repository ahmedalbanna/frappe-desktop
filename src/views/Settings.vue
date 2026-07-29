<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface LicenseInfo {
  key: string;
  machine_id: string;
  license_type: string;
  activated_at: string;
  expires_at: string;
  valid: boolean;
}

const license = ref<LicenseInfo | null>(null);
const licenseKey = ref("");
const offlineSig = ref("");
const activating = ref(false);
const activationError = ref("");
const activationMode = ref<"online" | "offline">("online");

onMounted(async () => {
  try {
    license.value = await invoke("get_license_status");
  } catch {}
});

async function activateOnline() {
  activating.value = true;
  activationError.value = "";
  try {
    license.value = await invoke("activate_license_online", {
      key: licenseKey.value,
    });
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
    license.value = await invoke("activate_license_offline", {
      key: licenseKey.value,
      signature: offlineSig.value,
    });
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
    license.value = await invoke("start_trial");
  } catch (e: any) {
    activationError.value = typeof e === "string" ? e : e.message || "فشل التفعيل التجريبي";
  } finally {
    activating.value = false;
  }
}

async function deactivate() {
  try {
    await invoke("deactivate_license");
    license.value = null;
    activationError.value = "";
  } catch (e: any) {
    activationError.value = typeof e === "string" ? e : e.message || "فشل إلغاء التفعيل";
  }
}

function copy(text: string) {
  navigator.clipboard.writeText(text);
}
</script>

<template>
  <div class="settings">
    <h1>الإعدادات</h1>

    <section class="card">
      <h2>الترخيص</h2>

      <!-- Active License -->
      <div v-if="license && license.valid" class="license-active">
        <div class="status-line">
          <span>الحالة:</span>
          <span class="badge configured">مُفعّل</span>
        </div>
        <div class="status-line">
          <span>النوع:</span>
          <span class="license-type">{{ license.license_type }}</span>
        </div>
        <div class="status-line">
          <span>تاريخ التفعيل:</span>
          {{ license.activated_at }}
        </div>
        <div class="status-line">
          <span>تاريخ الانتهاء:</span>
          {{ license.expires_at }}
        </div>
        <button class="btn btn-danger" @click="deactivate">إلغاء التفعيل</button>
      </div>

      <!-- Expired License -->
      <div v-else-if="license && license.license_type === 'expired'" class="license-expired">
        <p class="msg-error">انتهت صلاحية الترخيص في {{ license.expires_at }}</p>
        <p class="msg-hint">يرجى تجديد الترخيص للاستمرار في استخدام Frappe Desktop</p>
      </div>

      <!-- No License -->
      <div v-else class="license-form">
        <p class="msg-hint">لم يتم تفعيل الترخيص بعد. اختر طريقة التفعيل:</p>

        <div class="mode-tabs">
          <button
            :class="['tab', { active: activationMode === 'online' }]"
            @click="activationMode = 'online'"
          >تفعيل أونلاين</button>
          <button
            :class="['tab', { active: activationMode === 'offline' }]"
            @click="activationMode = 'offline'"
          >تفعيل يدوي</button>
          <button class="tab trial-tab" @click="startTrial" :disabled="activating">
            نسخة تجريبية ({{ 30 }} يوم)
          </button>
        </div>

        <div v-if="activationMode === 'online'" class="form-group">
          <label>مفتاح الترخيص</label>
          <input
            v-model="licenseKey"
            type="text"
            placeholder="XXXX-XXXX-XXXX-XXXX"
            :disabled="activating"
          />
          <button class="btn btn-primary" @click="activateOnline" :disabled="activating || !licenseKey">
            {{ activating ? 'جاري التفعيل...' : 'تفعيل' }}
          </button>
        </div>

        <div v-if="activationMode === 'offline'" class="form-group">
          <label>مفتاح الترخيص</label>
          <input v-model="licenseKey" type="text" placeholder="XXXX-XXXX-XXXX-XXXX" :disabled="activating" />
          <label>رمز التفعيل (من الموقع الرسمي)</label>
          <input v-model="offlineSig" type="text" placeholder="أدخل رمز التفعيل" :disabled="activating" />
          <button class="btn btn-primary" @click="activateOffline" :disabled="activating || !licenseKey || !offlineSig">
            {{ activating ? 'جاري التفعيل...' : 'تفعيل' }}
          </button>
        </div>

        <p v-if="activationError" class="msg-error">{{ activationError }}</p>
      </div>
    </section>

    <!-- Hardware ID -->
    <section class="card">
      <h2>معلومات الجهاز</h2>
      <div class="status-line">
        <span>Hardware ID:</span>
        <code class="mono">{{ license?.machine_id || '' }}</code>
        <button class="btn-xs" @click="copy(license?.machine_id || '')" v-if="license?.machine_id">📋</button>
      </div>
    </section>
  </div>
</template>

<style scoped>
.settings { max-width: 600px; margin: 0 auto; padding: 1.5rem; }
.settings h1 { font-size: 1.5rem; color: #1f2937; margin-bottom: 1.5rem; }
.card { background: white; border-radius: 10px; padding: 1.25rem; margin-bottom: 1.25rem; box-shadow: 0 1px 3px rgba(0,0,0,0.08); }
.card h2 { font-size: 1rem; color: #374151; margin-bottom: 1rem; }
.status-line { display: flex; align-items: center; gap: 0.75rem; margin-bottom: 0.5rem; font-size: 0.9rem; }
.status-line span { color: #6b7280; font-weight: 600; }
.badge { padding: 0.2rem 0.6rem; border-radius: 999px; font-size: 0.75rem; font-weight: 600; }
.configured { background: #dcfce7; color: #166534; }
.license-type { font-weight: 700; color: #2563eb; text-transform: capitalize; }
.mono { font-family: monospace; font-size: 0.8rem; background: #f3f4f6; padding: 0.25rem 0.5rem; border-radius: 4px; direction: ltr; display: inline-block; max-width: 300px; overflow: hidden; text-overflow: ellipsis; }
.btn-xs { padding: 0.25rem 0.5rem; border: none; border-radius: 3px; font-size: 0.75rem; cursor: pointer; background: #f3f4f6; }
.msg-hint { color: #6b7280; font-size: 0.85rem; margin-bottom: 1rem; }
.msg-error { color: #dc2626; font-size: 0.85rem; margin-top: 0.75rem; }
.mode-tabs { display: flex; gap: 0.5rem; margin-bottom: 1rem; }
.tab { padding: 0.4rem 1rem; border: 1px solid #e5e7eb; border-radius: 6px; background: #f9fafb; cursor: pointer; font-size: 0.8rem; font-weight: 600; }
.tab.active { background: #2563eb; color: white; border-color: #2563eb; }
.trial-tab { background: #fef3c7; border-color: #f59e0b; color: #92400e; }
.trial-tab:disabled { opacity: 0.5; cursor: not-allowed; }
.form-group { display: flex; flex-direction: column; gap: 0.5rem; margin-bottom: 0.75rem; }
.form-group label { font-size: 0.8rem; color: #6b7280; font-weight: 600; }
.form-group input { padding: 0.5rem; border: 1px solid #d1d5db; border-radius: 6px; font-size: 0.9rem; direction: ltr; text-align: left; }
.form-group input:focus { outline: none; border-color: #2563eb; box-shadow: 0 0 0 3px rgba(37,99,235,0.1); }
.btn { padding: 0.5rem 1rem; border: none; border-radius: 6px; font-size: 0.85rem; cursor: pointer; font-weight: 600; }
.btn:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-primary { background: #2563eb; color: white; }
.btn-danger { background: #dc2626; color: white; margin-top: 1rem; }
</style>
