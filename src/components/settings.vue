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
@import './settingsShell.css';
</style>
