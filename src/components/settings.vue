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

                  <div class="panel-actions">
                    <button class="settings-primary-btn" :disabled="!newUrl.trim()" @click="setPfp">Save picture</button>
                    <button class="settings-secondary-btn" @click="closeChangePfp">Reset</button>
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
			return this.newUrl.trim() || this.profilePic;
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
		},
		setPfp() {
			if (!this.newUrl.trim()) return;
			this.$emit('setOwnPfp', this.newUrl);
			this.pfpDialog = false;
			this.newUrl = "";
		}
	}
}
</script>

<style scoped>
/* overlay */
.settings-background,
.settings-overlay {
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
  background-color: #292222;
  width: min(1200px, 90vw);
  height: min(760px, 85vh);
  z-index: 10010;
  color: var(--main-font-color);
  border: 2px solid #4ebedb;
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
  border-bottom: 1px solid rgba(139, 63, 166, 0.55);
}

/* left panel */
.settings-sidebar {
  border-right: 2px solid #1b4f9c;
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
  border: 2px solid #4ebedb;
  background: #171313;
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
  color: #7fc8df;
  font-size: 0.9rem;
  cursor: pointer;
}

.sidebar-edit-btn:hover {
  color: #b8ebf8;
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
  border-bottom: 1px solid #8b3fa6;
  background: transparent;
  color: var(--main-font-color);
}
.sidebar-nav-item.active {
  background: #dfeef8;
  color: #111;
}

/* logout at bottom */
.sidebar-footer {
  margin-top: auto;
  border-top: 1px solid #8b3fa6;
  padding: 12px;
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
  color: rgba(255, 255, 255, 0.72);
}

.content-body,
.settings-panel {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.settings-card {
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(78, 190, 219, 0.22);
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
  color: rgba(255, 255, 255, 0.72);
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
  border: 2px solid #4ebedb;
  background: #171313;
  flex-shrink: 0;
}

.profile-fields {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.field-label {
  font-size: 0.9rem;
  color: rgba(255, 255, 255, 0.72);
}

.settings-input {
  width: 100%;
  padding: 12px 14px;
  border-radius: 12px;
  border: 1px solid rgba(78, 190, 219, 0.32);
  background: rgba(0, 0, 0, 0.2);
  color: var(--main-font-color);
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
  background: #4ebedb;
  color: #111;
}

.settings-secondary-btn {
  background: rgba(255, 255, 255, 0.08);
  color: var(--main-font-color);
}

.settings-danger-btn,
.sidebar-logout {
  background: rgba(208, 66, 66, 0.16);
  color: #ff9090;
}

.settings-primary-btn:hover {
  background: #79d4eb;
}

.settings-secondary-btn:hover {
  background: rgba(255, 255, 255, 0.14);
}

.settings-danger-btn:hover,
.sidebar-logout:hover {
  background: rgba(208, 66, 66, 0.26);
}

/* close button */
.settings-close-btn {
  position: absolute;
  top: 12px;
  right: 12px;
  z-index: 2;
  width: 34px;
  height: 34px;
  background: rgba(255, 255, 255, 0.08);
  color: var(--main-font-color);
}

.settings-close-btn:hover {
  background: rgba(255, 255, 255, 0.16);
}


</style>
