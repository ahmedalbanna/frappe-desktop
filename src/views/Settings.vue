<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface ServiceConfig {
  mariadb_port: number;
  redis_port: number;
  frappe_port: number;
  data_dir: string;
  mariadb_socket: string;
}

interface SetupStatus {
  configured: boolean;
  site_name: string | null;
  apps_installed: string[];
}

const config = ref<ServiceConfig | null>(null);
const setup = ref<SetupStatus | null>(null);
const hardwareId = ref("");

onMounted(async () => {
  try {
    config.value = await invoke("get_service_config");
  } catch {}
  try {
    setup.value = await invoke("check_setup_status");
  } catch {}
  try {
    hardwareId.value = await invoke("get_hardware_id");
  } catch {}
});

function copyHardwareId() {
  navigator.clipboard.writeText(hardwareId.value);
}
</script>

<template>
  <div class="settings">
    <h1>الإعدادات</h1>

    <section class="card">
      <h2>الخدمات</h2>
      <div v-if="config" class="config-grid">
        <div><span>MariaDB</span> {{ config.mariadb_port }}</div>
        <div><span>Redis</span> {{ config.redis_port }}</div>
        <div><span>Frappe</span> {{ config.frappe_port }}</div>
        <div class="full"><span>البيانات</span> {{ config.data_dir }}</div>
      </div>
    </section>

    <section class="card">
      <h2>الموقع</h2>
      <div v-if="setup">
        <div class="status-line">
          <span>الحالة:</span>
          <span v-if="setup.configured" class="badge configured">مُهيأ</span>
          <span v-else class="badge not-configured">غير مُهيأ</span>
        </div>
        <div v-if="setup.site_name" class="status-line">
          <span>الموقع:</span> {{ setup.site_name }}
        </div>
        <div v-if="setup.apps_installed.length" class="status-line">
          <span>التطبيقات:</span> {{ setup.apps_installed.join(", ") }}
        </div>
      </div>
    </section>

    <section class="card">
      <h2>الترخيص</h2>
      <div class="status-line">
        <span>Hardware ID:</span>
        <code class="hw-id">{{ hardwareId }}</code>
        <button class="btn-xs btn-start" @click="copyHardwareId">📋 نسخ</button>
      </div>
    </section>
  </div>
</template>

<style scoped>
.settings { max-width: 600px; margin: 0 auto; padding: 1.5rem; }
.settings h1 { font-size: 1.5rem; color: #1f2937; margin-bottom: 1.5rem; }
.card { background: white; border-radius: 10px; padding: 1.25rem; margin-bottom: 1.25rem; box-shadow: 0 1px 3px rgba(0,0,0,0.08); }
.card h2 { font-size: 1rem; color: #374151; margin-bottom: 1rem; }

.config-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 0.75rem; font-size: 0.85rem; }
.config-grid span { color: #6b7280; font-weight: 600; }
.full { grid-column: 1 / -1; }

.status-line { display: flex; align-items: center; gap: 0.75rem; margin-bottom: 0.5rem; font-size: 0.9rem; }
.status-line span { color: #6b7280; font-weight: 600; }

.badge { padding: 0.2rem 0.6rem; border-radius: 999px; font-size: 0.75rem; font-weight: 600; }
.configured { background: #dcfce7; color: #166534; }
.not-configured { background: #fee2e2; color: #991b1b; }

.hw-id { font-family: monospace; font-size: 0.8rem; background: #f3f4f6; padding: 0.25rem 0.5rem; border-radius: 4px; direction: ltr; display: inline-block; }
.btn-xs { padding: 0.25rem 0.5rem; border: none; border-radius: 3px; font-size: 0.75rem; cursor: pointer; }
.btn-start { background: #2563eb; color: white; }
</style>
