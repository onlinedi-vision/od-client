<template>
  <div class="settings-overlay">
    <div class="settings-modal">
	  <div class="settings-nav-buttons">
	    <button class="settings-nav-btn" @click="activeSection = 'main'" v-if="activeSection != 'main'">&#x2190;</button>
        <button class="settings-nav-btn" @click="$emit('closeSettings')">X</button>
	  </div>
      <aside class="settings-sidebar" v-if="!isMobile || activeSection === 'main'">
        <div class="sidebar-profile">
          <AvatarImg :src="profilePic" :width="64" :height="64" img-class="sidebar-avatar" />
          <div class="sidebar-user">
            <h3 class="sidebar-name">{{ userName }}</h3>
            <button class="sidebar-edit-btn" @click="openChangePfp">Edit profile</button>
          </div>
        </div>

        <nav class="sidebar-nav">
          <button v-for="section in sections" :key="section.key" class="sidebar-nav-item"
            :class="{ active: activeSection === section.key }" @click="activeSection = section.key">
            {{ section.label }}
          </button>
        </nav>

        <div class="sidebar-footer">
          <button class="sidebar-logout" @click="$emit('logOut')">Log out</button>
        </div>
      </aside>

      <main class="settings-content" v-if="!isMobile || activeSection != 'main'">
        <header class="content-header">
          <h2 class="content-title">{{ activeSectionLabel }}</h2>
          <p class="content-subtitle">{{ activeSectionDescription }}</p>
        </header>

        <section class="content-body">
          <div v-if="activeSection === 'customize'" class="settings-panel">
            <div class="settings-card">
              <div class="card-heading">
                <h3>Profile Picture</h3>
                <p>Use an image URL to update your avatar.</p>
              </div>

              <div class="profile-editor">
                <AvatarImg :src="previewProfilePic" :width="96" :height="96" img-class="profile-preview" />

                <div class="profile-fields">
                  <label class="field-label" for="pfp-url">Image URL</label>
                  <input id="pfp-url" v-model="newUrl" class="settings-input" type="text"
                    placeholder="https://example.com/avatar.png" />

                  <label class="field-label" for="pfp-file">Local image</label>
                  <div class="panel-actions">
                    <label class="settings-secondary-btn settings-file-btn" for="pfp-file">Choose file</label>
                    <span v-if="newFile" class="file-name">{{ newFile.name }}</span>
                    <input id="pfp-file" class="settings-file-input" type="file" accept="image/*,image/gif"
                      @change="onPfpFileChange" />
                  </div>

                  <div v-if="newFileUrl" class="profile-local-preview">
                    <img :src="newFileUrl" alt="preview" class="profile-local-preview-img" />
                    <button class="settings-secondary-btn" type="button" @click="clearFile">Remove</button>
                  </div>

                  <div class="panel-actions">
                    <button class="settings-primary-btn" type="button" :disabled="!canSavePfp" @click="setPfp">Save
                      picture</button>
                    <button class="settings-secondary-btn" type="button" @click="closeChangePfp">Reset</button>
                  </div>
                </div>
              </div>
            </div>

            <div class="settings-card">
              <div class="card-heading">
                <h3>Session</h3>
                <p>Close the modal or sign out of the current account.</p>
              </div>

              <div class="panel-actions">
                <button class="settings-secondary-btn" @click="$emit('closeSettings')">Close settings</button>
                <button class="settings-danger-btn" @click="$emit('logOut')">Log out</button>
              </div>
            </div>
          </div>

          <div v-else-if="activeSection === 'voice'" class="settings-panel">
            <div class="settings-card settings-placeholder">
              <h3>Voice & Video</h3>
              <p>This section is ready for microphone, speaker and camera controls when those settings exist in the app.
              </p>
            </div>
          </div>

          <div v-else-if="activeSection === 'keybinds'" class="settings-panel">
            <div class="settings-card settings-placeholder">
              <h3>Keybinds</h3>
              <p>Use this area later for shortcuts like push-to-talk, mute, or quick navigation bindings.</p>
            </div>
          </div>

          <div v-else-if="activeSection === 'notifications'" class="settings-panel">
            <div class="settings-card settings-placeholder">
              <h3>Notifications</h3>
              <p>Use this area later for message alerts, sound toggles and channel notification preferences.</p>
            </div>
          </div>

        </section>
      </main>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue';
const MOBILE_BREAKPOINT = 768;


const width = ref(window.innerWidth);
function onResize() { width.value = window.innerWidth };
onMounted(() => window.addEventListener('resize', onResize));
onUnmounted(() => window.removeEventListener('resize', onResize));

const isMobile = computed(() => width.value < MOBILE_BREAKPOINT);
</script>

<script>
import AvatarImg from './AvatarImg.vue'

export default {
  name: 'SettingsWindow',
  components: { AvatarImg },
  props: {
    userName: {
      type: String,
      required: true
    },
    profilePic: {
      type: String,
      required: true
    }
  },
  data() {
    return {
      pfpDialog: false,
      newUrl: "",
      newFile: null,
      newFileUrl: "",
      activeSection: "main",
      sections: [
        { key: "customize", label: "Customize" },
        { key: "voice", label: "Voice & Video" },
        { key: "keybinds", label: "Keybinds" },
        { key: "notifications", label: "Notifications" }
      ]
    };
  },
  computed: {
    activeSectionLabel() {
      const current = this.sections.find(s => s.key === this.activeSection);
      return current ? current.label : "Settings";
    },
    activeSectionDescription() {
      const descriptions = {
        customize: "Manage your profile image and session actions.",
        voice: "Reserved for audio input, output and device preferences.",
        keybinds: "Reserved for keyboard shortcuts and action bindings.",
        notifications: "Reserved for alert and notification behavior."
      };

      return descriptions[this.activeSection] || "Select a category of settings to get started.";
    },
    previewProfilePic() {
      return this.newFileUrl || this.newUrl.trim() || this.profilePic;
    },
    canSavePfp() {
      return Boolean(this.newUrl.trim() || this.newFile);
    }
  },
  methods: {
    openChangePfp() {
      this.activeSection = "customize";
      this.pfpDialog = true;
    },
    closeChangePfp() {
      this.pfpDialog = false;
      this.newUrl = "";
      this.clearFile();
    },
    setPfp() {
      if (!this.canSavePfp) return;
      this.$emit('setOwnPfp', { url: this.newUrl, file: this.newFile });
      this.pfpDialog = false;
      this.newUrl = "";
      this.clearFile();
    },
    onPfpFileChange(event) {
      const file = event?.target?.files?.[0];

      this.clearFile();
      if (!file) return;
      this.newFile = file;
      this.newFileUrl = URL.createObjectURL(file);
    },
    clearFile() {
      if (this.newFileUrl) {
        try { URL.revokeObjectURL(this.newFileUrl); } catch {/* Silently ignore errors from revokeObjectURL*/ }
      }
      this.newFile = null;
      this.newFileUrl = "";
      const input = document.getElementById("pfp-file");
      if (input) input.value = "";
    }
  }
}
</script>

<style scoped>
.settings-overlay {
  --settings-danger-bg: rgba(224, 122, 132, 0.16);
  --settings-danger-bg-hover: rgba(224, 122, 132, 0.26);

  background-color: rgba(0, 0, 0, 0.55);
  backdrop-filter: blur(6px);
  position: fixed;
  inset: 0;
  z-index: 10001;
  display: grid;
  place-items: center;
}

.settings-modal {
  background-color: var(--bg-chat);
  width: min(1200px, 90vw);
  height: min(760px, 85vh);
  z-index: 10010;
  color: var(--text-primary);
  border: 1px solid var(--border-subtle);
  border-radius: 14px;
  display: flex;
  flex-direction: row;
  position: relative;
  padding: 0;
  overflow: hidden;
}

.sidebar-profile {
  display: flex;
  align-items: center;
  gap: 12px;
  min-height: 104px;
  padding: 18px 16px;
  border-bottom: 1px solid var(--border-subtle);
}

.settings-sidebar {
  background-color: var(--bg-sidebar);
  border-right: 1px solid var(--border-subtle);
  display: flex;
  flex-direction: column;
  flex-basis: 280px;
  flex-grow: 1;
  min-width: 0;
}

:deep(.sidebar-avatar) {
  width: 64px;
  height: 64px;
  border-radius: 50%;
  border: 1px solid var(--border-subtle);
  box-sizing: border-box;
}

.sidebar-user {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.sidebar-name {
  margin: 0;
  font-size: 1rem;
  line-height: 1.2;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sidebar-edit-btn {
  align-self: flex-start;
  padding: 0;
  border: 0;
  background: transparent;
  color: var(--link);
  font-size: 0.9rem;
  cursor: pointer;
}

.sidebar-edit-btn:hover {
  color: var(--link-hover);
  background: transparent;
}

.sidebar-nav {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.sidebar-nav-item {
  height: 56px;
  text-align: left;
  padding: 0 16px;
  border: 0;
  border-bottom: 1px solid var(--border-subtle);
  background: transparent;
  color: var(--text-primary);
  cursor: pointer;
  transition: background-color 0.18s ease, color 0.18s ease, box-shadow 0.18s ease;
}

.sidebar-nav-item:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.sidebar-nav-item.active {
  background: var(--bg-active);
  color: var(--text-primary);
  font-weight: 700;
  box-shadow: inset 3px 0 0 var(--accent);
}

.sidebar-footer {
  margin-top: auto;
  border-top: 1px solid var(--border-subtle);
  padding: 0;
  min-height: 56px;
}

.settings-content {
  padding: 20px;
  overflow: auto;
  width: 100%;
  background-color: var(--bg-chat);
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.content-header {
  max-width: 560px;
  padding-right: 40px;
}

.content-title {
  margin: 0 0 8px;
  font-size: 1.8rem;
}

.content-subtitle {
  margin: 0;
  color: var(--text-muted);
}

.content-body,
.settings-panel {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.settings-card {
  background: var(--bg-sidebar);
  border: 1px solid var(--border-subtle);
  border-radius: 12px;
  padding: 20px;
}

.card-heading h3,
.settings-placeholder h3 {
  margin: 0 0 6px;
}

.card-heading p,
.settings-placeholder p {
  margin: 0;
  color: var(--text-muted);
}

.profile-editor {
  display: flex;
  align-items: center;
  gap: 20px;
  margin-top: 18px;
}

:deep(.profile-preview) {
  width: 96px;
  height: 96px;
  border-radius: 50%;
  border: 1px solid var(--border-subtle);
  box-sizing: border-box;
}

.profile-fields {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.profile-local-preview {
  display: flex;
  align-items: center;
  gap: 10px;
}

.profile-local-preview-img {
  max-width: 80px;
  max-height: 80px;
  border-radius: 8px;
  object-fit: cover;
  border: 1px solid var(--border-subtle);
}

.field-label {
  font-size: 0.9rem;
  color: var(--text-muted);
}

.file-name {
  color: var(--text-muted);
  font-size: 0.9rem;
}

.settings-input {
  width: 100%;
  max-width: 100%;
  box-sizing: border-box;
  padding: 12px 14px;
  border-radius: 10px;
  border: 1px solid var(--border-subtle);
  background: var(--bg-input);
  color: var(--text-primary);
}

.settings-input:focus {
  outline: none;
  border-color: var(--accent-muted);
}

.panel-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  margin-top: 6px;
}

.settings-primary-btn,
.settings-secondary-btn,
.settings-danger-btn,
.sidebar-logout,
.settings-nav-btn {
  border: 0;
  border-radius: 10px;
  cursor: pointer;
  transition: background-color 0.18s ease, color 0.18s ease, opacity 0.18s ease, border-color 0.18s ease;
}

.settings-primary-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.settings-primary-btn,
.settings-secondary-btn,
.settings-danger-btn,
.sidebar-logout {
  padding: 10px 14px;
  font-weight: 600;
}

.settings-primary-btn {
  background: var(--accent);
  color: #1a1214;
}

.settings-primary-btn:hover:not(:disabled) {
  background: var(--accent-hover);
  opacity: 1;
}

.settings-secondary-btn {
  background: var(--bg-rail);
  color: var(--text-primary);
  border: 1px solid var(--border-subtle);
}

.settings-secondary-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.settings-file-btn {
  display: inline-flex;
  align-items: center;
}

.settings-danger-btn,
.sidebar-logout {
  background: var(--settings-danger-bg);
  color: var(--danger);
}

.sidebar-logout {
  width: 100%;
  height: 100%;
  min-height: 56px;
  border-radius: 0;
  text-align: left;
  padding: 0 16px;
}

.settings-danger-btn:hover,
.sidebar-logout:hover {
  background: var(--settings-danger-bg-hover);
  color: var(--danger);
}

.settings-file-input {
  display: none;
}

.settings-nav-buttons {
  position: absolute;
  top: 12px;
  right: 12px;
  z-index: 2;
}

.settings-nav-btn {
  width: 34px;
  height: 34px;
  padding: 0;
  margin: 0 6px 0 6px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  text-align: center;
  line-height: 1;
  font-size: 16px;
  background: var(--bg-sidebar);
  color: var(--text-primary);
  border: 1px solid var(--border-subtle);
}

.settings-nav-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}
</style>
