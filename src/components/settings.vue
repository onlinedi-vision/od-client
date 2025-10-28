<template>
<div class="settings">
<div class="settings-titlebar">
<div><h1>Settings</h1></div>
<button @click="$emit('closeSettings')" class="closebutton">X</button>
</div>
<div class="settings-container">
<div style="flex:1">
<div class="image-wrapper">
<div class="image-container"><img src='https://media1.tenor.com/m/viIU4ICp1N8AAAAd/dance.gif' class='large-icon' />
<button class="edit-pfp">Edit</button>
</div>
</div>
<div class="name-change">
<h2>{{userName}}</h2><button class="menu-button">Change</button>
</div>
</div>
<div style="flex:2; background-color: #282424; border: solid 2px #141414;">
</div>
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