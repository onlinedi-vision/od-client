<template>
  <div class="settings-overlay">
    <div class="settings-modal">
      <button class="settings-close-btn" @click="$emit('closeSettings')">X</button>

      <aside class="settings-sidebar">
        <div class="sidebar-profile">
          <img :src="profilePic" class="sidebar-avatar" />
          <div class="sidebar-user">
            <h3 class="sidebar-name">{{ userName }}</h3>
            <button class="sidebar-edit-btn" @click="openChangePfp">Edit profile</button>
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
          <button class="sidebar-logout" @click="$emit('logOut')">Log out</button>
        </div>
      </aside>

      <main class="settings-content">
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
                <img :src="previewProfilePic" class="profile-preview" />

                <div class="profile-fields">
                  <label class="field-label" for="pfp-url">Image URL</label>
                  <input
                    id="pfp-url"
                    v-model="newUrl"
                    class="settings-input"
                    type="text"
                    placeholder="https://example.com/avatar.png"
                  />

                  <label class="field-label" for="pfp-file">Local image</label>
                  <div class="panel-actions">
                    <label class="settings-secondary-btn settings-file-btn" for="pfp-file">Choose file</label>
                    <span v-if="newFile" class="file-name">{{ newFile.name }}</span>
                    <input
                      id="pfp-file"
                      class="settings-file-input"
                      type="file"
                      accept="image/*,image/gif"
                      @change="onPfpFileChange"
                    />
                  </div>

                  <div v-if="newFileUrl" class="profile-local-preview">
                    <img :src="newFileUrl" alt="preview" class="profile-local-preview-img" />
                    <button class="settings-secondary-btn" type="button" @click="clearFile">Remove</button>
                  </div>

                  <div class="panel-actions">
                    <button class="settings-primary-btn" type="button" :disabled="!canSavePfp" @click="setPfp">Save picture</button>
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
              <p>This section is ready for microphone, speaker and camera controls when those settings exist in the app.</p>
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


<script>
export default {
	name: 'SettingsWindow',
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
			activeSection: "customize",
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

			return descriptions[this.activeSection] || "Settings";
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
				try { URL.revokeObjectURL(this.newFileUrl); } catch (_) {}
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
/* overlay */
.settings-background,
.settings-overlay {
  --settings-bg: var(--normal-foreground-element-color);
  --settings-surface: var(--dark-foreground-element-color);
  --settings-text: var(--main-font-color);
  --settings-app-bg: var(--background-color);
  background-color: rgba(0, 0, 0, 0.45);
  position: fixed;
  inset: 0;
  z-index: 10001;
  /* remove blur */
  backdrop-filter: none;
  display: grid;
  place-items: center;
}

/* modal */
.settings,
.settings-modal {
  background-color: var(--settings-bg);
  width: min(1200px, 90vw);
  height: min(760px, 85vh);
  z-index: 10010;
  color: var(--settings-text);
  border: 2px solid var(--settings-text);
  display: grid;
  grid-template-columns: 280px 1fr;
  position: relative;
  padding: 0;
}

.sidebar-profile {
  display: flex;
  align-items: center;
  gap: 12px;
  min-height: 104px;
  padding: 18px 16px;
  border-bottom: 1px solid var(--settings-text);
}

/* left panel */
.settings-sidebar {
  border-right: 2px solid var(--settings-text);
  display: flex;
  flex-direction: column;
}

.sidebar-avatar {
  width: 64px;
  height: 64px;
  border-radius: 50%;
  object-fit: cover;
  flex-shrink: 0;
  display: block;
  border: 2px solid var(--settings-text);
  background: var(--settings-surface);
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
  color: var(--settings-text);
  font-size: 0.9rem;
  cursor: pointer;
}

.sidebar-edit-btn:hover {
  color: #fff;
}

/* section list */
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
  border-bottom: 1px solid var(--settings-text);
  background: transparent;
  color: var(--settings-text);
  cursor: pointer;
  transition: background-color 0.18s ease, color 0.18s ease, box-shadow 0.18s ease;
}

.sidebar-nav-item:hover {
  background: var(--main-font-color);
  color: #fff;
}

.sidebar-nav-item.active {
  background: var(--settings-app-bg);
  color: #fff;
  font-weight: 700;
  box-shadow: inset 4px 0 0 #fff;
}

/* logout at bottom */
.sidebar-footer {
  margin-top: auto;
  border-top: 1px solid var(--settings-text);
  padding: 0;
  min-height: 56px;
}

/* right panel */
.settings-right,
.settings-content {
  padding: 20px;
  overflow: auto;
}

.settings-content {
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
  color: var(--settings-text);
}

.content-body,
.settings-panel {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.settings-card {
  background: var(--settings-surface);
  border: 1px solid var(--settings-text);
  border-radius: 16px;
  padding: 20px;
}

.card-heading h3,
.settings-placeholder h3 {
  margin: 0 0 6px;
}

.card-heading p,
.settings-placeholder p {
  margin: 0;
  color: var(--settings-text);
}

.profile-editor {
  display: flex;
  align-items: center;
  gap: 20px;
  margin-top: 18px;
}

.profile-preview {
  width: 96px;
  height: 96px;
  border-radius: 50%;
  object-fit: cover;
  border: 2px solid var(--settings-text);
  background: var(--settings-surface);
  flex-shrink: 0;
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
}

.field-label {
  font-size: 0.9rem;
  color: var(--settings-text);
}

.file-name {
  color: var(--settings-text);
  font-size: 0.9rem;
}

.settings-input {
  width: 100%;
  max-width: 100%;
  box-sizing: border-box;
  padding: 12px 14px;
  border-radius: 12px;
  border: 1px solid var(--settings-text);
  background: var(--settings-app-bg);
  color: var(--settings-text);
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
.settings-close-btn {
  border: 0;
  border-radius: 10px;
  cursor: pointer;
  transition: background-color 0.18s ease, color 0.18s ease, opacity 0.18s ease;
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
  background: var(--settings-text);
  color: #fff;
}

.settings-secondary-btn {
  background: var(--settings-surface);
  color: var(--settings-text);
}

.settings-file-btn {
  display: inline-flex;
  align-items: center;
}

.settings-danger-btn,
.sidebar-logout {
  background: rgba(208, 66, 66, 0.16);
  color: #ff9090;
}

.sidebar-logout {
  width: 100%;
  height: 100%;
  min-height: 56px;
  border-radius: 0;
  text-align: left;
}

.settings-primary-btn:hover {
  opacity: 0.85;
}

.settings-secondary-btn:hover {
  background: var(--settings-bg);
  color: #fff;
}

.settings-danger-btn:hover,
.sidebar-logout:hover {
  background: rgba(208, 66, 66, 0.26);
}

.settings-file-input {
  display: none;
}

/* close button */
.settings-close-btn {
  position: absolute;
  top: 12px;
  right: 12px;
  z-index: 2;
  width: 34px;
  height: 34px;
  padding: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  text-align: center;
  line-height: 1;
  font-size: 16px;
  background: var(--settings-surface);
  color: var(--settings-text);
  box-shadow: 0 0 6px rgba(128, 128, 128, 0.5);
}

.settings-close-btn:hover {
  background: var(--settings-bg);
  color: #fff;
}


</style>
