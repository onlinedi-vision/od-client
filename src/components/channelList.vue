<template>
  <div class="chanels">
    <div class="server-header" @click="$emit('showSID')">
      <h3>{{ serverName }}</h3>
    </div>

    <template v-if="done && currentServer?.storedChannels?.length">
	  <template v-for="(div, index) in currentServer.storedChannels.sort(compareChannels)"
        :key="index">
	  <div class="channel_container">	
      <button
        @click="$emit('changeChannel', div.channelTag)"
        class="channel_button main"
        :class="{ active: div.channelTag === textChannel }"
      >
	  <template v-if="confirmDelete==index">
		Confirm delete?
	  </template>
	  <template v-else>
        # {{ div.channelTag }}
	  </template>
      </button>
	  <template v-if="confirmDelete==index">
	    <button class="channel_options always_on" @click="deleteChannel(currentServer.serverID, div.channelTag)">
		<i class="pi pi-check"/>
	  </button>
	  <button class="channel_options always_on" @click="cancelDelete()">
		  <i class="pi pi-times" />
	  </button>
	  </template>
	  <template v-else-if="optionsOpenIndex==index">
	  <button class="channel_options always_on" @click="askDelete(index)">
		  <i class="pi pi-trash"/>
	  </button>
	  <button class="channel_options always_on" @click="closeOptions()">
		  <i class="pi pi-undo" />
	  </button>
      </template>
	  <template v-else>
	  <button class="channel_options">
		  <i class="pi pi-phone" />
	  </button>
	  <button class="channel_options" @click="openOptions(index)">
		  <i class="pi pi-ellipsis-h" />
	  </button>
	  </template>
	  </div>
	  </template>
    </template>

    <button @click="$emit('createChannel', serverID)" class="channel_button add-channel">
      <b>+</b>
    </button>
  </div>
</template>

<script>
export default {
  name: "ChannelList",
  props: {
    appState: Array,
    serverID: String,
    textChannel: String,
    done: Boolean,
    userServers: Array,
  },
  data(){
    return {
      optionsOpenIndex: -1,
	  confirmDelete: -1,
    };
  },
  computed: {
    currentServer() {
      return this.appState.find(obj => obj.serverID === this.serverID);
    },
    serverName() {
      const sv = this.userServers.find(obj => obj.serverID === this.serverID);
      return sv ? sv.name : "Unknown Server";
    },
  },
  methods: {
	compareChannels(a, b){
	  if(a.channelTag > b.channelTag)
	    return 1;
	  if(b.channelTag > a.channelTag)
	    return -1;
	  return 0;
	},
	openOptions(idx){
	  this.optionsOpenIndex = idx;
	},
	closeOptions(){
	  this.optionsOpenIndex = -1;
	},
	askDelete(idx){
	  this.confirmDelete = idx;
	},
	cancelDelete(){
	  this.confirmDelete = -1;
	},
	deleteChannel(sid, channelTag){
	  this.$emit('deleteChannel', sid, channelTag);
	  this.confirmDelete = -1;
	  this.optionsOpenIndex = -1;
	},
  },
};
</script>

<style>
.chanels {
  display: flex;
  flex-shrink: 0;
  flex-direction: column;
}
.server-header {
  border-bottom: 1px solid var(--border-subtle);
  height: 60px;
  background: var(--bg-rail);
  border-radius: 14px 14px 0 0;
}
.server-header h3 {
  margin: 5px 0 0 0;
}
.channel_button,
.channel_options {
  padding: 6px 5px;
  border: none;
  background-color: var(--bg-sidebar);
  color: var(--text-primary);
  cursor: pointer;
  text-align: left;
  transition: background-color 0.2s ease, color 0.2s ease, opacity 0.15s ease;
}

.channel_options {
  padding: 6px 3px;
  opacity: 0;
  pointer-events: none;
}

.channel_button:hover,
.channel_button:hover ~ .channel_options {
  background-color: var(--bg-hover);
}

.channel_button:hover ~ .channel_options {
  color: var(--text-muted);
  animation: fadeIn 0.2s forwards;
  pointer-events: auto;
}

.channel_button.active {
  background-color: var(--bg-active);
  color: var(--text-primary);
  box-shadow: inset 3px 0 0 var(--accent);
}

.channel_button.active ~ .channel_options {
  background-color: var(--bg-active);
  opacity: 1;
  pointer-events: auto;
}
.channel_button.main {
  flex-grow: 1;
  text-overflow: ellipsis;
  overflow: hidden;
  white-space: nowrap;
}
.channel_button.add-channel {
  position: relative;
  text-align: center;
  padding: 6px 0;
}
.channel_container{
  display: flex;
  flex-direction: row;
  justify-content: space-between;
}
.channel_container:hover .channel_options {
  animation: fadeIn 0.2s forwards;
  pointer-events: auto;
}

@keyframes fadeIn {
  to {
    opacity: 1;
  }
}

.channel_options:hover {
  opacity: 1;
  pointer-events: auto;
  background-color: var(--bg-hover);
  color: var(--text-primary);
}
.always_on{
  opacity: 1;
  pointer-events: auto;
}
</style>
