<template>
<div class="settings">
<div class="settings-titlebar">
<div><h3>{{userName}}'s profile</h3></div>
<button @click="$emit('closeSettings')" class="closebutton">X</button>
</div>
<button @click="$emit('logOut')" class="logout">Log out</button>
</div>
</template>

<script>
export default {
	name: 'SettingsWindow',
	props: {
		userName: {
			type: String,
			required: true
		}
	},
	data(){
		return {};
	}
}
</script>