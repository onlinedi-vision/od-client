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
<div v-if="pfpDialog" class="login" style="display:flex;flex-direction:row; left:200px; top:100px; height:50px; width:280px">
        <input id="greet-input" v-model="newUrl" style="width:200px" placeholder="profile pic URL"/>
        <button id="send" type="submit" style="width:80px" @click="setPfp()">Set</button>
		<button id="send" style="width:80px" @click="closeChangePfp()">Cancel</button>
</div>
</div>
</div>
<div class="name-change">
<h2 style="margin: 2px 0 2px 0">{{userName}}</h2><button class="menu-button"><h4 style="margin: 2px 2px 2px 2px">Change name</h4></button>
</div>
</div>
<div class="settings-right">
<div>
<button @click="$emit('logOut')" class="logout"><h4 style="margin: 0 0 0 0">Log out</h4></button>
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
	data(){
		return {
			pfpDialog: false,
			newUrl: "",
		};
	},
	methods: {
		openChangePfp() {
			this.pfpDialog = true;
		},
		closeChangePfp() {
			this.pfpDialog = false;
			this.newUrl = "";
		},
		setPfp() {
			this.$emit('setOwnPfp', this.newUrl);
			this.pfpDialog = false;
		}
	}
}
</script>
