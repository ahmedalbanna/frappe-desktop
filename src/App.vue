<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface ProcessInfo {
  state: string;
  pid: number | null;
  healthy: boolean;
}

interface ServiceConfig {
  mariadb_port: number;
  redis_port: number;
  frappe_port: number;
  data_dir: string;
  mariadb_socket: string;
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

let pollTimer: ReturnType<typeof setInterval> | null = null;

async function fetchStatus() {
  try {
    const data: ProcessInfo[] = await invoke("get_service_status");
    statuses.value.mariadb = data[0];
    statuses.value.redis = data[1];
    statuses.value.frappe = data[2];
  } catch (e) {
    console.error("Failed to fetch status:", e);
  }
}

async function fetchConfig() {
  try {
    config.value = await invoke("get_service_config");
  } catch (e) {
    console.error("Failed to fetch config:", e);
  }
}

async function fetchLogs(name: ServiceName) {
  try {
    logs.value[name] = await invoke("get_service_logs", { name });
  } catch (e) {
    console.error("Failed to fetch logs:", e);
  }
}

async function startAll() {
  try {
    await invoke("start_all_services");
    frappeUrl.value = await invoke<string>("get_frappe_url");
    await fetchStatus();
  } catch (e) {
    console.error("Failed to start services:", e);
  }
}

async function stopAll() {
  try {
    await invoke("stop_all_services");
    frappeUrl.value = "";
    await fetchStatus();
  } catch (e) {
    console.error("Failed to stop services:", e);
  }
}

async function startService(name: ServiceName) {
  try {
    await invoke("start_service", { name });
    await fetchStatus();
  } catch (e) {
    console.error(`Failed to start ${name}:`, e);
  }
}

async function stopService(name: ServiceName) {
  try {
    await invoke("stop_service", { name });
    await fetchStatus();
  } catch (e) {
    console.error(`Failed to stop ${name}:`, e);
  }
}

async function restartService(name: ServiceName) {
  try {
    await invoke("restart_service", { name });
    await fetchStatus();
  } catch (e) {
    console.error(`Failed to restart ${name}:`, e);
  }
}

function selectLog(name: ServiceName) {
  activeLog.value = name;
  fetchLogs(name);
}

onMounted(async () => {
  await fetchConfig();
  await fetchStatus();
  await fetchLogs(activeLog.value);

  pollTimer = setInterval(() => {
    fetchStatus();
  }, 5000);
});

onUnmounted(() => {
  if (pollTimer) clearInterval(pollTimer);
});
</script>

<template>
  <div class="app">
    <header class="header">
      <h1>Frappe Desktop</h1>
      <p class="subtitle">منصة الأعمال المتكاملة على سطح المكتب</p>
    </header>

    <main class="main">
      <!-- Services -->
      <section class="card">
        <h2>حالة الخدمات</h2>
        <div class="status-grid">
          <div
            v-for="name in serviceNames"
            :key="name"
            class="status-card"
            :class="statuses[name].state"
          >
            <div class="service-header">
              <span class="service-icon">{{ name === "mariadb" ? "🗄️" : name === "redis" ? "⚡" : "🌐" }}</span>
              <span class="service-name">{{ name }}</span>
            </div>
            <div class="service-meta">
              <span :class="['badge', statuses[name].state]">
                {{ statuses[name].state }}
              </span>
              <span v-if="statuses[name].healthy" class="badge healthy">✓ صحية</span>
              <span v-else-if="statuses[name].state === 'running'" class="badge unhealthy">✗ غير صحية</span>
            </div>
            <div v-if="statuses[name].pid" class="pid">PID: {{ statuses[name].pid }}</div>
            <div class="service-actions">
              <button
                v-if="statuses[name].state !== 'running'"
                class="btn-sm btn-start"
                @click="startService(name)"
              >▶ تشغيل</button>
              <button
                v-if="statuses[name].state === 'running'"
                class="btn-sm btn-stop"
                @click="stopService(name)"
              >■ إيقاف</button>
              <button
                v-if="statuses[name].state === 'running'"
                class="btn-sm btn-restart"
                @click="restartService(name)"
              >↻ إعادة</button>
            </div>
          </div>
        </div>

        <div class="global-actions">
          <button class="btn btn-start" @click="startAll">▶ تشغيل الكل</button>
          <button class="btn btn-stop" @click="stopAll">■ إيقاف الكل</button>
        </div>

        <div v-if="frappeUrl" class="url-bar">
          <span>التطبيق متاح على:</span>
          <a :href="frappeUrl" target="_blank">{{ frappeUrl }}</a>
        </div>
      </section>

      <!-- Config -->
      <section v-if="config" class="card">
        <h2>الإعدادات</h2>
        <div class="config-grid">
          <div><span>MariaDB:</span> {{ config.mariadb_port }}</div>
          <div><span>Redis:</span> {{ config.redis_port }}</div>
          <div><span>Frappe:</span> {{ config.frappe_port }}</div>
          <div class="full-width"><span>البيانات:</span> {{ config.data_dir }}</div>
        </div>
      </section>

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
    </main>
  </div>
</template>

<style>
* { margin: 0; padding: 0; box-sizing: border-box; }
body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  background: #f0f2f5; color: #1a1a2e; direction: rtl;
}
.app { max-width: 960px; margin: 0 auto; padding: 1.5rem; }
.header { text-align: center; margin-bottom: 1.5rem; }
.header h1 { font-size: 1.75rem; color: #2563eb; }
.subtitle { color: #6b7280; margin-top: 0.25rem; font-size: 0.9rem; }

.card {
  background: white; border-radius: 10px; padding: 1.25rem;
  margin-bottom: 1.25rem; box-shadow: 0 1px 3px rgba(0,0,0,0.08);
}
.card h2 { font-size: 1rem; color: #374151; margin-bottom: 1rem; }

.status-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 1rem; }
.status-card {
  background: #f9fafb; border-radius: 8px; padding: 1rem;
  border: 1px solid #e5e7eb; text-align: center;
}
.status-card.running { border-color: #86efac; background: #f0fdf4; }
.status-card.stopped { border-color: #fca5a5; background: #fef2f2; }

.service-header { display: flex; align-items: center; justify-content: center; gap: 0.5rem; margin-bottom: 0.75rem; }
.service-icon { font-size: 1.5rem; }
.service-name { font-weight: 700; text-transform: uppercase; font-size: 0.9rem; color: #374151; }

.service-meta { display: flex; gap: 0.5rem; justify-content: center; margin-bottom: 0.5rem; }
.badge {
  padding: 0.2rem 0.6rem; border-radius: 999px; font-size: 0.75rem;
  font-weight: 600; text-transform: uppercase;
}
.badge.running { background: #dcfce7; color: #166534; }
.badge.stopped { background: #fee2e2; color: #991b1b; }
.badge.error { background: #fef3c7; color: #92400e; }
.badge.healthy { background: #dbeafe; color: #1e40af; }
.badge.unhealthy { background: #fef3c7; color: #92400e; }

.pid { font-size: 0.7rem; color: #9ca3af; margin-bottom: 0.5rem; }

.service-actions { display: flex; gap: 0.35rem; justify-content: center; flex-wrap: wrap; }
.btn-sm {
  padding: 0.3rem 0.6rem; border: none; border-radius: 4px;
  font-size: 0.75rem; cursor: pointer; font-weight: 600;
}
.btn-start { background: #2563eb; color: white; }
.btn-stop { background: #dc2626; color: white; }
.btn-restart { background: #d97706; color: white; }

.global-actions { display: flex; gap: 0.75rem; justify-content: center; margin-top: 1rem; }
.btn {
  padding: 0.6rem 1.5rem; border: none; border-radius: 6px;
  font-size: 0.9rem; cursor: pointer; font-weight: 600;
}
.btn-start { background: #2563eb; color: white; }
.btn-stop { background: #dc2626; color: white; }

.url-bar { text-align: center; margin-top: 0.75rem; font-size: 0.85rem; }
.url-bar a { color: #2563eb; font-weight: 600; text-decoration: none; }

.config-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 0.75rem; font-size: 0.85rem; }
.config-grid span { color: #6b7280; font-weight: 600; }
.full-width { grid-column: 1 / -1; }

.log-tabs { display: flex; gap: 0.5rem; margin-bottom: 0.75rem; }
.tab {
  padding: 0.35rem 1rem; border: 1px solid #e5e7eb; border-radius: 6px;
  background: #f9fafb; cursor: pointer; font-size: 0.8rem; font-weight: 600;
}
.tab.active { background: #2563eb; color: white; border-color: #2563eb; }

.log-viewer {
  background: #1e293b; color: #e2e8f0; border-radius: 6px;
  padding: 0.75rem; max-height: 300px; overflow-y: auto;
  font-family: 'Monaco', 'Menlo', monospace; font-size: 0.75rem; line-height: 1.5;
}
.log-empty { color: #64748b; text-align: center; padding: 2rem; }
.log-line { white-space: pre-wrap; word-break: break-all; }
</style>
