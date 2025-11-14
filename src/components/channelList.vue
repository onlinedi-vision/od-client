<template>
  <div class="chanels">
    <div class="server-header" @click="$emit('showSID')">
      <h3>{{ serverName }}</h3>
    </div>

    <template v-if="done && currentServer?.storedChannels?.length">
      <button
        v-for="(div, index) in currentServer.storedChannels.sort(compareChannels)"
        :key="index"
        @click="$emit('changeChannel', div.channelTag)"
        class="channel_button"
        :class="{ active: div.channelTag === textChannel }"
      >
        # {{ div.channelTag }}
      </button>
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
	}
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
  padding: 6px 10px;
  border: none;
  background: none;
  color: #ccc;
  cursor: pointer;
  text-align: left;
  transition: all 0.2s ease;
}
.channel_button:hover {
  background-color: #a87678;
  color: white;
}
.channel_button.active {
  background-color: #a87678;
  color: white;
}
.channel_button.add-channel {
  position: relative;
  text-align: center;
  padding: 6px 0;
}
</style>
