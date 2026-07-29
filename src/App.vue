<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import FirstRunWizard from "./views/FirstRunWizard.vue";
import Dashboard from "./views/Dashboard.vue";
import Settings from "./views/Settings.vue";

type View = "loading" | "wizard" | "dashboard" | "settings";

const currentView = ref<View>("loading");

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

onMounted(checkSetup);
</script>

<template>
  <div class="app-shell">
    <!-- Loading -->
    <div v-if="currentView === 'loading'" class="loading-screen">
      <div class="spinner"></div>
      <p>جاري التحميل...</p>
    </div>

    <!-- First-Run Wizard -->
    <FirstRunWizard
      v-if="currentView === 'wizard'"
      @complete="goTo('dashboard')"
    />

    <!-- Dashboard -->
    <div v-if="currentView === 'dashboard' || currentView === 'settings'">
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
