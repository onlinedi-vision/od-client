<template>
  <div class="settings-overlay" @click.self="$emit('close')">
    <div class="settings-modal" @click.stop>
      <div class="settings-nav-buttons">
        <button
          class="settings-nav-btn"
          @click="activeSection = 'main'"
          v-if="isMobile && activeSection !== 'main'"
        >
          &#x2190;
        </button>
        <button class="settings-nav-btn" @click="$emit('close')">X</button>
      </div>

      <aside class="settings-sidebar" v-if="!isMobile || activeSection === 'main'">
        <div class="sidebar-profile">
          <AvatarImg
            :src="serverImg"
            :width="64"
            :height="64"
            img-class="sidebar-avatar"
          />
          <div class="sidebar-user">
            <h3 class="sidebar-name">{{ serverName }}</h3>
            <p class="sidebar-server-id">{{ serverId }}</p>
          </div>
        </div>

        <nav class="sidebar-nav">
          <button
            v-for="section in sections"
            :key="section.key"
            class="sidebar-nav-item"
            :class="{ active: activeSection === section.key }"
            @click="activeSection = section.key"
          >
            {{ section.label }}
          </button>
        </nav>

        <div class="sidebar-footer">
          <button class="sidebar-logout settings-secondary-btn" type="button" @click="$emit('close')">
            Close
          </button>
        </div>
      </aside>

      <main class="settings-content" v-if="!isMobile || activeSection !== 'main'">
        <header class="content-header">
          <h2 class="content-title">{{ activeSectionLabel }}</h2>
          <p class="content-subtitle">{{ activeSectionDescription }}</p>
        </header>

        <section class="content-body">
          <div v-if="activeSection === 'frontend'" class="settings-panel">
            <div class="settings-card">
              <div class="card-heading">
                <h3>Frontend appearance</h3>
                <p>Chat background, message avatars, and message font for this server.</p>
              </div>

              <div class="profile-fields form-stack">
                <label class="field-label" for="fe-bg-color">Background color</label>
                <input
                  id="fe-bg-color"
                  v-model="fields.backgroundColor"
                  class="settings-input"
                  type="text"
                  placeholder="#0b090a"
                />

                <label class="field-label" for="fe-bg-image">Background image URL</label>
                <input
                  id="fe-bg-image"
                  v-model="fields.backgroundImage"
                  class="settings-input"
                  type="text"
                  placeholder="https://… (png, gif, jpg)"
                />

                <label class="field-label" for="fe-icons">Message icons</label>
                <select id="fe-icons" v-model="fields.messageIcons" class="settings-input">
                  <option value="square">Square</option>
                  <option value="circle">Circle</option>
                </select>

                <label class="field-label" for="fe-font">Message font</label>
                <input
                  id="fe-font"
                  v-model="fields.messageFont"
                  class="settings-input"
                  type="text"
                  placeholder="Inter, sans-serif"
                />

                <p v-if="applyError" class="form-error">{{ applyError }}</p>
                <p v-if="applySuccess" class="form-success">{{ applySuccess }}</p>

                <div class="panel-actions">
                  <button
                    class="settings-primary-btn"
                    type="button"
                    :disabled="applying"
                    @click="onApply"
                  >
                    {{ applying ? 'Applying…' : 'Apply' }}
                  </button>
                  <button class="settings-secondary-btn" type="button" @click="resetFields">
                    Reset
                  </button>
                </div>
              </div>
            </div>
          </div>

          <div v-else-if="activeSection === 'overview'" class="settings-panel">
            <div class="settings-card settings-placeholder">
              <h3>Overview</h3>
              <p>Server overview and metadata will live here later.</p>
            </div>
          </div>

          <div v-else-if="activeSection === 'members'" class="settings-panel">
            <div class="settings-card settings-placeholder">
              <h3>Members</h3>
              <p>Member management and roles will live here later.</p>
            </div>
          </div>
        </section>
      </main>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import AvatarImg from './AvatarImg.vue'
import { feConfigToEditorFields } from '../serverFeConfig.js'

const MOBILE_BREAKPOINT = 768

const props = defineProps({
  serverId: { type: String, required: true },
  serverName: { type: String, required: true },
  serverImg: { type: String, default: '' },
  feConfig: { type: Object, required: true },
  applyError: { type: String, default: '' },
  applySuccess: { type: String, default: '' },
  applying: { type: Boolean, default: false },
})

const emit = defineEmits(['close', 'apply'])

const width = ref(window.innerWidth)
function onResize() {
  width.value = window.innerWidth
}
onMounted(() => window.addEventListener('resize', onResize))
onUnmounted(() => window.removeEventListener('resize', onResize))

const isMobile = computed(() => width.value < MOBILE_BREAKPOINT)
const activeSection = ref('main')
const fields = ref(feConfigToEditorFields(props.feConfig))

const sections = [
  { key: 'frontend', label: 'Frontend' },
  { key: 'overview', label: 'Overview' },
  { key: 'members', label: 'Members' },
]

watch(
  () => props.feConfig,
  (c) => {
    fields.value = feConfigToEditorFields(c)
  },
  { deep: true }
)

watch(
  () => props.serverId,
  () => {
    activeSection.value = 'main'
    fields.value = feConfigToEditorFields(props.feConfig)
  }
)

const activeSectionLabel = computed(() => {
  const s = sections.find((x) => x.key === activeSection.value)
  return s ? s.label : 'Server settings'
})

const activeSectionDescription = computed(() => {
  const d = {
    frontend: 'Customize how this server looks in the chat client.',
    overview: 'Reserved for server summary and details.',
    members: 'Reserved for member and permission settings.',
  }
  return d[activeSection.value] || 'Server settings'
})

function resetFields() {
  fields.value = feConfigToEditorFields(props.feConfig)
}

function onApply() {
  emit('apply', { ...fields.value })
}
</script>

<script>
export default {
  name: 'ServerSettingsWindow',
}
</script>

<style scoped>
@import './settingsShell.css';

.sidebar-server-id {
  margin: 0;
  font-size: 0.8rem;
  color: var(--text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.form-stack {
  margin-top: 16px;
}

.form-error {
  margin: 8px 0 0;
  color: var(--danger);
  font-size: 0.9rem;
}

.form-success {
  margin: 8px 0 0;
  color: var(--accent);
  font-size: 0.9rem;
}

.sidebar-footer .settings-secondary-btn {
  width: 100%;
  min-height: 56px;
  border-radius: 0;
  border: 0;
  text-align: left;
  padding: 0 16px;
}
</style>
