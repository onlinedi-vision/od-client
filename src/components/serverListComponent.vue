<template>
  <div class="chanels">
    <div 
      class="server-header" 
      @click="$emit('showSID')" 
      style="border-bottom: 2px solid #141414; height: 60px;"
    >
      <h3 style="margin-bottom: 0px; margin-top: 5px;">
        {{ serverName }}
      </h3>
    </div>
    <template v-if="done && currentServer?.storedChannels">
      <template 
        v-for="(div, index) in currentServer.storedChannels" 
        :key="index"
      >
        <button
          @click="$emit('changeChannel', div.channelTag)"
          class="channel_button"
          :style="div.channelTag === textChannel 
            ? 'background-color: #a87678; color: white' 
            : ''"
        >
          # {{ div.channelTag }}
        </button>
      </template>
    </template>
    <button 
      @click="$emit('createChannel', serverID)" 
      class="channel_button" 
      style="position: relative; text-align: center; padding-left: 0px; padding-right: 0px;"
    >
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
    userServers: Array
  },
  computed: {
    currentServer() {
      return this.appState.find(obj => obj.serverID === this.serverID);
    },
    serverName() {
      const sv = this.userServers.find(obj => obj.serverID === this.serverID);
      return sv ? sv.name : "Unknown Server";
    }
  }
};
</script>
<style scoped>
.chanels {
  display: flex;
  flex-direction: column;
}
.channel_button {
  padding: 6px 10px;
  border: none;
  background: none;
  color: #ccc;
  cursor: pointer;
  text-align: left;
}
.channel_button:hover {
  background-color: #333;
  color: white;
}
</style>
