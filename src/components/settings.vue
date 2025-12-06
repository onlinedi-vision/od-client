<template>
<div class="settings">
<div class="settings-titlebar">
<div><h1>Settings</h1></div>
<!-- <button @click="$emit('closeSettings')" class="closebutton">X</button> -->
</div>
<div class="settings-container">
<div style="flex:1">
<div class="image-wrapper">
<div class="image-container"><img v-bind:src='profilePic' class='large-icon' />
<button @click="openChangePfp()" class="edit-pfp"><h4>Edit</h4></button>
<div v-if="pfpDialog" class="pfp-dialog">
  <label class="pfp-label">From URL</label>
  <input class="pfp-input" v-model="newUrl" placeholder="https://example.com/pic.png"/>

  <label class="pfp-label">From device</label>
  <input id="pfp-upload" class="pfp-file" type="file" accept="image/*,image/gif" @change="onPfpFileChange" />
  <label for="pfp-upload" class="pfp-file-button">Choose file</label>
  <div v-if="newFile" class="pfp-file-name">{{ newFile?.name }}</div>

  <div v-if="newFileUrl" class="pfp-preview">
    <img :src="newFileUrl" alt="preview" class="pfp-preview-img" />
    <button type="button" class="pfp-remove" @click="clearFile()">Remove</button>
  </div>

  <div class="pfp-actions">
    <button type="button" class="pfp-btn" @click="setPfp()">Set</button>
    <button type="button" class="pfp-btn" @click="closeChangePfp()">Cancel</button>
  </div>
</div>

</div>
</div>
<div class="name-change">
<h2 style="margin: 2px 0 2px 0">{{userName}}</h2><button class="menu-button"><h4 style="margin: 2px 2px 2px 2px">Change name</h4></button>
</div>
</div>
<div class="settings-right">
	<div class="settings-grid">
		<button @click="$emit('logOut')" class="settings-card"><h4 style="margin: 0 0 0 0">Change Name</h4></button>
		<button @click="$emit('closeSettings')" class="settings-card"><h4 style="margin: 0 0 0 0">Change Picture</h4></button>
		<button @click="$emit('logOut')" class="settings-card"><h4 style="margin: 0 0 0 0">Log Out</h4></button>
		<button @click="$emit('closeSettings')" class="settings-card"><h4 style="margin: 0 0 0 0">Return</h4></button>
	</div>
</div>
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
			newFileUrl: ""
		};
		},
		methods: {
		openChangePfp() { this.pfpDialog = true; },
		closeChangePfp() {
			this.pfpDialog = false;
			this.newUrl = "";
			this.clearFile();
		},
		onPfpFileChange(e) {
			const file = e?.target?.files?.[0];
			this.clearFile();
			if (file) {
			this.newFile = file;
			this.newFileUrl = URL.createObjectURL(file);
			}
		},
		clearFile() {
			if (this.newFileUrl) {
			try { URL.revokeObjectURL(this.newFileUrl); } catch (_) {}
			}
			this.newFile = null;
			this.newFileUrl = "";
			const input = document.getElementById("pfp-upload");
			if (input) input.value = "";
		},
		setPfp() {
			this.$emit('setOwnPfp', { url: this.newUrl, file: this.newFile });
			this.newUrl = "";
			this.clearFile();
			this.pfpDialog = false;
		}
	}

}
</script>
<style scoped>
	.pfp-dialog {
		position: absolute;
		top: 120px;
		left: 50%;
		transform: translateX(-50%);
		width: 360px;
		background: var(--dark-foreground-element-color);
		border-radius: 10px;
		padding: 14px;
		display: flex;
		flex-direction: column;
		gap: 10px;
		z-index: 10020;
	}
	.pfp-label { font-size: 12px; color: #aaa; text-align: left; }
	.pfp-input { width: 100%; box-sizing: border-box; }
	.pfp-file { display: none; }
	.pfp-file-button {
		display: inline-block; padding: 8px 10px; background: var(--normal-foreground-element-color);
		color: var(--main-font-color); border-radius: 6px; cursor: pointer; text-align: center;
	}
	.pfp-file-name { font-size: 12px; color: var(--main-font-color); word-break: break-all; }
	.pfp-preview { display: flex; align-items: center; gap: 10px; }
	.pfp-preview-img { max-width: 80px; max-height: 80px; border-radius: 8px; }
	.pfp-remove, .pfp-btn {
		padding: 8px 10px; background: var(--normal-foreground-element-color) !important;
		color: var(--main-font-color) !important; border: none; border-radius: 6px; cursor: pointer;
	}
	.pfp-remove:hover, .pfp-btn:hover { background: var(--main-font-color) !important; color: #fff !important; }
	.pfp-actions { display: flex; gap: 8px; }
	.pfp-actions .pfp-btn { flex: 1; }
</style>
