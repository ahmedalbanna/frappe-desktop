<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

export interface ProcessInfo {
  state: string;
  pid: number | null;
  healthy: boolean;
}

export interface ServiceConfig {
  mariadb_port: number;
  redis_port: number;
  frappe_port: number;
  data_dir: string;
  mariadb_socket: string;
}

interface SystemInfo {
  os: string;
  cpu_cores: number;
  memory_gb: string;
  disk_free_gb: string;
}

const serviceNames = ["mariadb", "redis", "frappe"] as const;
type ServiceName = (typeof serviceNames)[number];

const statuses = ref<Record<ServiceName, ProcessInfo>>({
  mariadb: { state: "stopped", pid: null, healthy: false },
  redis: { state: "stopped", pid: null, healthy: false },
  frappe: { state: "stopped", pid: null, healthy: false },
});

const config = ref<ServiceConfig | null>(null);
const logs = ref<Record<ServiceName, string[]>>({
  mariadb: [],
  redis: [],
  frappe: [],
});
const activeLog = ref<ServiceName>("mariadb");
const frappeUrl = ref("");
const siteName = ref("");

let pollTimer: ReturnType<typeof setInterval> | null = null;

async function fetchStatus() {
  try {
    const data: ProcessInfo[] = await invoke("get_service_status");
    statuses.value.mariadb = data[0];
    statuses.value.redis = data[1];
    statuses.value.frappe = data[2];
  } catch (e) {
    console.error("fetchStatus:", e);
  }
}

async function fetchConfig() {
  try {
    config.value = await invoke("get_service_config");
  } catch (e) {
    console.error("fetchConfig:", e);
  }
}

async function fetchLogs(name: ServiceName) {
  try {
    logs.value[name] = await invoke("get_service_logs", { name });
  } catch (e) {
    console.error("fetchLogs:", e);
  }
}

async function startAll() {
  try {
    await invoke("start_all_services");
    frappeUrl.value = await invoke<string>("get_frappe_url");
    await fetchStatus();
  } catch (e) {
    console.error("startAll:", e);
  }
}

async function stopAll() {
  try {
    await invoke("stop_all_services");
    frappeUrl.value = "";
    await fetchStatus();
  } catch (e) {
    console.error("stopAll:", e);
  }
}

async function startService(name: ServiceName) {
  try { await invoke("start_service", { name }); await fetchStatus(); }
  catch (e) { console.error(`startService ${name}:`, e); }
}

async function stopService(name: ServiceName) {
  try { await invoke("stop_service", { name }); await fetchStatus(); }
  catch (e) { console.error(`stopService ${name}:`, e); }
}

async function restartService(name: ServiceName) {
  try { await invoke("restart_service", { name }); await fetchStatus(); }
  catch (e) { console.error(`restartService ${name}:`, e); }
}

function selectLog(name: ServiceName) {
  activeLog.value = name;
  fetchLogs(name);
}

function openFrappe() {
  window.open(frappeUrl.value, "_blank");
}

onMounted(async () => {
  await fetchConfig();
  await fetchStatus();
  await fetchLogs(activeLog.value);

  try {
    const setup = await invoke<{ site_name?: string }>("check_setup_status");
    if (setup.site_name) siteName.value = setup.site_name;
  } catch {}

  pollTimer = setInterval(fetchStatus, 5000);
});

onUnmounted(() => {
  if (pollTimer) clearInterval(pollTimer);
});
</script>

<template>
  <div class="dashboard">
    <!-- Header -->
    <div class="dash-header">
      <div>
        <h1>لوحة التحكم</h1>
        <p v-if="siteName" class="site-badge">الموقع: {{ siteName }}</p>
      </div>
      <div class="header-actions">
        <button v-if="frappeUrl" class="btn btn-accent" @click="openFrappe">🚀 فتح Frappe</button>
      </div>
    </div>

    <!-- Service Cards -->
    <section class="card">
      <div class="card-title-row">
        <h2>الخدمات</h2>
        <div class="global-actions">
          <button class="btn-sm btn-start" @click="startAll">▶ تشغيل الكل</button>
          <button class="btn-sm btn-stop" @click="stopAll">■ إيقاف الكل</button>
        </div>
      </div>
      <div class="status-grid">
        <div
          v-for="name in serviceNames"
          :key="name"
          class="status-card"
          :class="statuses[name].state"
        >
          <div class="service-header">
            <span class="service-icon">{{ { mariadb: "🗄️", redis: "⚡", frappe: "🌐" }[name] }}</span>
            <span class="service-name">{{ name }}</span>
          </div>
          <div class="service-meta">
            <span :class="['badge', statuses[name].state]">{{ statuses[name].state }}</span>
            <span v-if="statuses[name].healthy" class="badge healthy">✓ صحي</span>
            <span v-else-if="statuses[name].state === 'running'" class="badge unhealthy">✗ غير صحي</span>
          </div>
          <div v-if="statuses[name].pid" class="pid">PID {{ statuses[name].pid }}</div>
          <div class="service-actions">
            <button v-if="statuses[name].state !== 'running'" class="btn-xs btn-start" @click="startService(name)">▶</button>
            <button v-if="statuses[name].state === 'running'" class="btn-xs btn-stop" @click="stopService(name)">■</button>
            <button v-if="statuses[name].state === 'running'" class="btn-xs btn-restart" @click="restartService(name)">↻</button>
          </div>
        </div>
      </div>
    </section>

    <!-- Quick Actions + Info -->
    <div class="dash-grid-2">
      <section class="card">
        <h2>إجراءات سريعة</h2>
        <div class="quick-actions">
          <button class="action-btn" disabled>
            <span class="action-icon">💾</span>
            <span>نسخ احتياطي</span>
          </button>
          <button class="action-btn" disabled>
            <span class="action-icon">⚙️</span>
            <span>الإعدادات</span>
          </button>
          <button class="action-btn" disabled>
            <span class="action-icon">📋</span>
            <span>السجلات</span>
          </button>
        </div>
      </section>

      <section v-if="config" class="card">
        <h2>معلومات النظام</h2>
        <div class="info-list">
          <div><span>MariaDB</span> port {{ config.mariadb_port }}</div>
          <div><span>Redis</span> port {{ config.redis_port }}</div>
          <div><span>Frappe</span> port {{ config.frappe_port }}</div>
          <div class="full"><span>البيانات</span> {{ config.data_dir }}</div>
        </div>
      </section>
    </div>

    <!-- Logs -->
    <section class="card">
      <h2>السجلات</h2>
      <div class="log-tabs">
        <button
          v-for="name in serviceNames"
          :key="name"
          :class="['tab', { active: activeLog === name }]"
          @click="selectLog(name)"
        >{{ name }}</button>
      </div>
      <div class="log-viewer">
        <div v-if="logs[activeLog].length === 0" class="log-empty">لا توجد سجلات</div>
        <div v-for="(line, i) in logs[activeLog]" :key="i" class="log-line">{{ line }}</div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.dashboard { max-width: 960px; margin: 0 auto; padding: 1.5rem; }
.dash-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 1.5rem; }
.dash-header h1 { font-size: 1.5rem; color: #1f2937; }
.site-badge { font-size: 0.85rem; color: #2563eb; font-weight: 600; margin-top: 0.25rem; }
.header-actions { display: flex; gap: 0.75rem; }

.card { background: white; border-radius: 10px; padding: 1.25rem; margin-bottom: 1.25rem; box-shadow: 0 1px 3px rgba(0,0,0,0.08); }
.card h2 { font-size: 1rem; color: #374151; margin-bottom: 1rem; }
.card-title-row { display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; }
.card-title-row h2 { margin-bottom: 0; }

.status-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 1rem; }
.status-card { background: #f9fafb; border-radius: 8px; padding: 1rem; border: 1px solid #e5e7eb; text-align: center; }
.status-card.running { border-color: #86efac; background: #f0fdf4; }
.status-card.stopped { border-color: #fca5a5; background: #fef2f2; }
.service-header { display: flex; align-items: center; justify-content: center; gap: 0.5rem; margin-bottom: 0.75rem; }
.service-icon { font-size: 1.5rem; }
.service-name { font-weight: 700; text-transform: uppercase; font-size: 0.9rem; color: #374151; }
.service-meta { display: flex; gap: 0.5rem; justify-content: center; margin-bottom: 0.5rem; }
.badge { padding: 0.2rem 0.6rem; border-radius: 999px; font-size: 0.75rem; font-weight: 600; text-transform: uppercase; }
.badge.running { background: #dcfce7; color: #166534; }
.badge.stopped { background: #fee2e2; color: #991b1b; }
.badge.error { background: #fef3c7; color: #92400e; }
.badge.healthy { background: #dbeafe; color: #1e40af; }
.badge.unhealthy { background: #fef3c7; color: #92400e; }
.pid { font-size: 0.7rem; color: #9ca3af; margin-bottom: 0.5rem; }

.service-actions { display: flex; gap: 0.35rem; justify-content: center; }
.global-actions { display: flex; gap: 0.5rem; }
.btn-sm { padding: 0.35rem 0.75rem; border: none; border-radius: 4px; font-size: 0.75rem; cursor: pointer; font-weight: 600; }
.btn-xs { padding: 0.25rem 0.5rem; border: none; border-radius: 3px; font-size: 0.75rem; cursor: pointer; font-weight: 700; }
.btn-start { background: #2563eb; color: white; }
.btn-stop { background: #dc2626; color: white; }
.btn-restart { background: #d97706; color: white; }
.btn-accent { background: #059669; color: white; padding: 0.5rem 1rem; border: none; border-radius: 6px; font-size: 0.85rem; cursor: pointer; font-weight: 600; }

.dash-grid-2 { display: grid; grid-template-columns: 1fr 1fr; gap: 1.25rem; }
.quick-actions { display: flex; flex-direction: column; gap: 0.5rem; }
.action-btn {
  display: flex; align-items: center; gap: 0.75rem; padding: 0.75rem;
  border: 1px solid #e5e7eb; border-radius: 6px; background: #f9fafb;
  cursor: not-allowed; font-size: 0.9rem; color: #9ca3af; width: 100%;
}
.action-icon { font-size: 1.25rem; }

.info-list { display: flex; flex-direction: column; gap: 0.5rem; font-size: 0.85rem; }
.info-list span { color: #6b7280; font-weight: 600; margin-left: 0.5rem; }
.info-list .full { grid-column: 1 / -1; }

.log-tabs { display: flex; gap: 0.5rem; margin-bottom: 0.75rem; }
.tab { padding: 0.35rem 1rem; border: 1px solid #e5e7eb; border-radius: 6px; background: #f9fafb; cursor: pointer; font-size: 0.8rem; font-weight: 600; }
.tab.active { background: #2563eb; color: white; border-color: #2563eb; }

.log-viewer { background: #1e293b; color: #e2e8f0; border-radius: 6px; padding: 0.75rem; max-height: 250px; overflow-y: auto; font-family: 'Monaco','Menlo',monospace; font-size: 0.75rem; line-height: 1.5; }
.log-empty { color: #64748b; text-align: center; padding: 2rem; }
.log-line { white-space: pre-wrap; word-break: break-all; }
</style>
