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
        # {{ div.channelTag }}
      </button>
	  <template v-if="this.optionsOpenIndex==index">
	  <button class="channel_options always_on" @click="$emit('deleteChannel', currentServer.serverID, div.channelTag)">
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
  },
};
</script>

<style>
.chanels {
  display: flex;
  flex-direction: column;
}
.server-header {
  border-bottom: 2px solid #141414;
  height: 60px;
}
.server-header h3 {
  margin: 5px 0 0 0;
}
.channel_button {
  padding: 6px 5px;
  border: none;
  background: none;
  color: var(--main-font-color);
  cursor: pointer;
  text-align: left;
  transition: all 0.2s ease;
}
.channel_button:hover ~ .channel_options {
  background-color: var(--main-font-color);
  color: white;
  opacity: 1;
  pointer-events: auto;
}
.channel_button.active {
  background-color: --main-font-color;
  color: white;
}
.channel_button.active ~ .channel_options {
  background-color: --main-font-color;
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
  opacity: 1;
  pointer-events: auto;
}
.channel_options{
  padding: 6px 3px;
  border: none;
  background: none;
  color: var(--main-font-color);
  cursor: pointer;
  text-align: left;
  transition: all 0.15s ease;
  opacity: 0;
  pointer-events: none;
}
.channel_options:hover {
  opacity: 1;
  pointer-events: auto;
}
.always_on{
  opacity: 1;
  pointer-events: auto;
}
</style>
