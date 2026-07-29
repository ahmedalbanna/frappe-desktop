<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface ServiceStatus {
  mariadb: string;
  redis: string;
  frappe: string;
}

const status = ref<ServiceStatus>({
  mariadb: "stopped",
  redis: "stopped",
  frappe: "stopped",
});
const frappeUrl = ref("");

async function checkStatus() {
  status.value = await invoke<ServiceStatus>("get_service_status");
}

async function startServices() {
  await invoke("start_all_services");
  await checkStatus();
  frappeUrl.value = await invoke<string>("get_frappe_url");
}

async function stopServices() {
  await invoke("stop_all_services");
  await checkStatus();
}

onMounted(async () => {
  await checkStatus();
});
</script>

<template>
  <div class="app">
    <header class="header">
      <h1>Frappe Desktop</h1>
      <p class="subtitle">منصة الأعمال المتكاملة على سطح المكتب</p>
    </header>

    <main class="main">
      <section class="services">
        <h2>حالة الخدمات</h2>
        <div class="status-grid">
          <div class="status-card" v-for="(state, name) in status" :key="name">
            <span class="service-name">{{ name }}</span>
            <span :class="['badge', state]">{{ state }}</span>
          </div>
        </div>
      </section>

      <section class="actions">
        <button class="btn btn-start" @click="startServices" :disabled="status.mariadb === 'running'">
          تشغيل الخدمات
        </button>
        <button class="btn btn-stop" @click="stopServices" :disabled="status.mariadb === 'stopped'">
          إيقاف الخدمات
        </button>
      </section>

      <section v-if="frappeUrl" class="launch">
        <p>الخدمات تعمل على: <a :href="frappeUrl" target="_blank">{{ frappeUrl }}</a></p>
      </section>
    </main>
  </div>
</template>

<style>
* { margin: 0; padding: 0; box-sizing: border-box; }
body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f5f5f5; color: #333; direction: rtl; }
.app { max-width: 800px; margin: 0 auto; padding: 2rem; }
.header { text-align: center; margin-bottom: 2rem; }
.header h1 { font-size: 2rem; color: #1a73e8; }
.subtitle { color: #666; margin-top: 0.5rem; }
.status-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 1rem; margin: 1rem 0; }
.status-card { background: white; border-radius: 8px; padding: 1rem; text-align: center; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }
.service-name { display: block; font-weight: 600; margin-bottom: 0.5rem; text-transform: uppercase; font-size: 0.9rem; }
.badge { padding: 0.25rem 0.75rem; border-radius: 12px; font-size: 0.85rem; }
.badge.running { background: #e6f4ea; color: #137333; }
.badge.stopped { background: #fce8e6; color: #c5221f; }
.badge.error { background: #fef7e0; color: #ea8600; }
.actions { display: flex; gap: 1rem; justify-content: center; margin: 2rem 0; }
.btn { padding: 0.75rem 2rem; border: none; border-radius: 6px; font-size: 1rem; cursor: pointer; font-weight: 600; }
.btn:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-start { background: #1a73e8; color: white; }
.btn-stop { background: #d93025; color: white; }
.launch { text-align: center; margin-top: 1rem; }
.launch a { color: #1a73e8; text-decoration: none; font-weight: 600; }
</style>
