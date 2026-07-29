<script setup lang="ts">
import { ref, reactive } from "vue";
import { invoke } from "@tauri-apps/api/core";

const emit = defineEmits<{
  complete: [];
}>();

const step = ref(1);
const loading = ref(false);
const error = ref("");
const progressLog = ref<string[]>([]);

interface WizardData {
  siteName: string;
  adminPassword: string;
  adminEmail: string;
  firstName: string;
  lastName: string;
  companyName: string;
  abbreviation: string;
  country: string;
  currency: string;
  apps: string[];
}

const data = reactive<WizardData>({
  siteName: "frappe.local",
  adminPassword: "admin",
  adminEmail: "admin@example.com",
  firstName: "Admin",
  lastName: "User",
  companyName: "My Company",
  abbreviation: "MC",
  country: "United States",
  currency: "USD",
  apps: ["erpnext"],
});

const availableApps = [
  { id: "erpnext", label: "ERPNext", desc: "ERP and business management" },
  { id: "hr", label: "HR", desc: "Human Resources" },
  { id: "payroll", label: "Payroll", desc: "Payroll management" },
  { id: "crm", label: "CRM", desc: "Customer Relationship Management" },
  { id: "manufacturing", label: "Manufacturing", desc: "Manufacturing module" },
  { id: "selling", label: "Selling", desc: "Sales management" },
];

async function runSetup() {
  loading.value = true;
  error.value = "";
  progressLog.value = [];

  try {
    progressLog.value.push("🔧 Creating Frappe site...");
    await invoke("create_site", {
      siteName: data.siteName,
      adminPassword: data.adminPassword,
    });

    if (data.apps.length > 0) {
      progressLog.value.push(`📦 Installing apps: ${data.apps.join(", ")}...`);
      await invoke("install_apps", {
        siteName: data.siteName,
        apps: data.apps,
      });
    }

    progressLog.value.push("🏢 Setting up company information...");
    await invoke("setup_company", {
      siteName: data.siteName,
      data: {
        company_name: data.companyName,
        abbreviation: data.abbreviation,
        country: data.country,
        currency: data.currency,
      },
    });

    progressLog.value.push("✅ Setup complete!");
    await new Promise((r) => setTimeout(r, 1000));
    emit("complete");
  } catch (e: any) {
    error.value = typeof e === "string" ? e : e.message || "Setup failed";
    progressLog.value.push(`❌ Error: ${error.value}`);
  } finally {
    loading.value = false;
  }
}

function toggleApp(appId: string) {
  const idx = data.apps.indexOf(appId);
  if (idx > -1) {
    data.apps.splice(idx, 1);
  } else {
    data.apps.push(appId);
  }
}

function nextStep() {
  error.value = "";
  if (step.value < steps.length) step.value++;
}

function prevStep() {
  error.value = "";
  if (step.value > 1) step.value--;
}

const steps = [
  { num: 1, title: "مرحباً", subtitle: "الإعداد الأولي" },
  { num: 2, title: "الموقع", subtitle: "إعداد موقع Frappe" },
  { num: 3, title: "المستخدم", subtitle: "معلومات المدير" },
  { num: 4, title: "الشركة", subtitle: "معلومات الشركة" },
  { num: 5, title: "التطبيقات", subtitle: "اختيار التطبيقات" },
  { num: 6, title: "تثبيت", subtitle: "جاري الإعداد" },
];
</script>

<template>
  <div class="wizard">
    <header class="wizard-header">
      <h1>Frappe Desktop</h1>
      <p class="subtitle">الإعداد الأولي — {{ steps[step - 1].subtitle }}</p>
    </header>

    <!-- Progress bar -->
    <div class="progress-bar">
      <div
        v-for="s in steps.slice(0, 5)"
        :key="s.num"
        :class="['step-dot', { active: s.num === step, done: s.num < step }]"
      >{{ s.num }}</div>
    </div>

    <div class="wizard-body">
      <!-- Step 1: Welcome -->
      <div v-if="step === 1" class="wizard-step">
        <h2>مرحباً بك في Frappe Desktop!</h2>
        <p>سيقوم هذا المعالج بإعداد موقع Frappe الأول الخاص بك. ستتمكن من:</p>
        <ul>
          <li>إنشاء موقع Frappe جديد</li>
          <li>تثبيت التطبيقات (ERPNext وغيرها)</li>
          <li>إعداد معلومات الشركة</li>
        </ul>
        <p class="hint">سيستغرق الإعداد من 5 إلى 10 دقائق حسب سرعة الاتصال.</p>
      </div>

      <!-- Step 2: Site Setup -->
      <div v-if="step === 2" class="wizard-step">
        <h2>إعداد الموقع</h2>
        <div class="form-group">
          <label>اسم الموقع</label>
          <input v-model="data.siteName" type="text" placeholder="frappe.local" />
          <span class="hint">سيتم الوصول للموقع عبر http://{{ data.siteName }}:8000</span>
        </div>
        <div class="form-group">
          <label>كلمة سر المدير</label>
          <input v-model="data.adminPassword" type="password" placeholder="admin" />
        </div>
      </div>

      <!-- Step 3: Admin User -->
      <div v-if="step === 3" class="wizard-step">
        <h2>معلومات المدير</h2>
        <div class="form-row">
          <div class="form-group">
            <label>الاسم الأول</label>
            <input v-model="data.firstName" type="text" placeholder="Admin" />
          </div>
          <div class="form-group">
            <label>الاسم الأخير</label>
            <input v-model="data.lastName" type="text" placeholder="User" />
          </div>
        </div>
        <div class="form-group">
          <label>البريد الإلكتروني</label>
          <input v-model="data.adminEmail" type="email" placeholder="admin@example.com" />
        </div>
      </div>

      <!-- Step 4: Company -->
      <div v-if="step === 4" class="wizard-step">
        <h2>معلومات الشركة</h2>
        <div class="form-group">
          <label>اسم الشركة</label>
          <input v-model="data.companyName" type="text" placeholder="My Company" />
        </div>
        <div class="form-row">
          <div class="form-group">
            <label>الاختصار</label>
            <input v-model="data.abbreviation" type="text" placeholder="MC" maxlength="5" />
          </div>
          <div class="form-group">
            <label>الدولة</label>
            <input v-model="data.country" type="text" placeholder="United States" />
          </div>
          <div class="form-group">
            <label>العملة</label>
            <input v-model="data.currency" type="text" placeholder="USD" />
          </div>
        </div>
      </div>

      <!-- Step 5: Apps -->
      <div v-if="step === 5" class="wizard-step">
        <h2>اختيار التطبيقات</h2>
        <p>اختر التطبيقات التي تريد تثبيتها:</p>
        <div class="app-list">
          <div
            v-for="app in availableApps"
            :key="app.id"
            :class="['app-item', { selected: data.apps.includes(app.id) }]"
            @click="toggleApp(app.id)"
          >
            <span class="app-check">{{ data.apps.includes(app.id) ? '✓' : '' }}</span>
            <div>
              <strong>{{ app.label }}</strong>
              <span class="app-desc">{{ app.desc }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Step 6: Progress -->
      <div v-if="step === 6" class="wizard-step">
        <h2>جاري الإعداد...</h2>
        <div class="progress-log">
          <div v-for="(line, i) in progressLog" :key="i" class="log-line">{{ line }}</div>
          <div v-if="loading" class="log-line blinking">⏳ جاري العمل...</div>
        </div>
        <div v-if="error" class="error-box">{{ error }}</div>
      </div>
    </div>

    <div class="wizard-footer">
      <button v-if="step > 1 && step < 6" class="btn btn-outline" @click="prevStep" :disabled="loading">
        → السابق
      </button>
      <button
        v-if="step < 5"
        class="btn btn-primary"
        @click="nextStep"
      >
        التالي ←
      </button>
      <button v-if="step === 5" class="btn btn-primary" @click="step = 6; runSetup()" :disabled="loading">
        {{ loading ? 'جاري التثبيت...' : 'ابدأ التثبيت' }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.wizard { max-width: 600px; margin: 0 auto; padding: 2rem 1.5rem; }
.wizard-header { text-align: center; margin-bottom: 1.5rem; }
.wizard-header h1 { font-size: 1.5rem; color: #2563eb; }
.wizard-header .subtitle { color: #6b7280; font-size: 0.9rem; }

.progress-bar { display: flex; justify-content: center; gap: 0.5rem; margin-bottom: 2rem; }
.step-dot {
  width: 32px; height: 32px; border-radius: 50%; display: flex;
  align-items: center; justify-content: center; font-weight: 700;
  font-size: 0.85rem; background: #e5e7eb; color: #9ca3af;
}
.step-dot.active { background: #2563eb; color: white; }
.step-dot.done { background: #86efac; color: #166534; }

.wizard-body { min-height: 300px; }
.wizard-step h2 { font-size: 1.25rem; margin-bottom: 1rem; color: #1f2937; }
.wizard-step p { color: #6b7280; line-height: 1.6; margin-bottom: 1rem; }
.wizard-step ul { padding-right: 1.5rem; margin-bottom: 1rem; line-height: 2; }
.hint { font-size: 0.8rem; color: #9ca3af; margin-top: 0.25rem; display: block; }

.form-group { margin-bottom: 1rem; }
.form-group label { display: block; font-weight: 600; font-size: 0.9rem; margin-bottom: 0.35rem; color: #374151; }
.form-group input {
  width: 100%; padding: 0.6rem 0.75rem; border: 1px solid #d1d5db;
  border-radius: 6px; font-size: 0.9rem; outline: none; direction: ltr;
}
.form-group input:focus { border-color: #2563eb; box-shadow: 0 0 0 2px rgba(37,99,235,0.1); }
.form-row { display: flex; gap: 1rem; }
.form-row .form-group { flex: 1; }

.app-list { display: flex; flex-direction: column; gap: 0.5rem; }
.app-item {
  display: flex; align-items: center; gap: 0.75rem; padding: 0.75rem;
  border: 1px solid #e5e7eb; border-radius: 8px; cursor: pointer;
}
.app-item:hover { border-color: #93c5fd; }
.app-item.selected { border-color: #2563eb; background: #eff6ff; }
.app-check {
  width: 24px; height: 24px; border-radius: 4px; border: 2px solid #d1d5db;
  display: flex; align-items: center; justify-content: center; font-weight: 700;
  flex-shrink: 0;
}
.app-item.selected .app-check { background: #2563eb; color: white; border-color: #2563eb; }
.app-desc { display: block; font-size: 0.8rem; color: #9ca3af; }

.progress-log {
  background: #1e293b; color: #e2e8f0; border-radius: 8px;
  padding: 1rem; max-height: 300px; overflow-y: auto;
  font-family: monospace; font-size: 0.85rem; line-height: 1.8;
}
.blinking::after { content: ''; animation: blink 1s step-end infinite; }
@keyframes blink { 50% { opacity: 0; } }
.error-box { background: #fef2f2; color: #991b1b; padding: 0.75rem; border-radius: 6px; margin-top: 1rem; }

.wizard-footer { display: flex; gap: 0.75rem; justify-content: center; margin-top: 2rem; }
.btn {
  padding: 0.6rem 1.5rem; border: none; border-radius: 6px;
  font-size: 0.9rem; cursor: pointer; font-weight: 600;
}
.btn:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-primary { background: #2563eb; color: white; }
.btn-outline { background: transparent; border: 1px solid #d1d5db; color: #374151; }
</style>
