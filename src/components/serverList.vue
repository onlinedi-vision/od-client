<template>
  <div class="server_list">

    <ServerListItem
      :userServers="userServers"
      :appState="appState"
      :serverID="serverID"
      :textChannel="textChannel"

      @click="$emit('changeServer', sv.serverID)"
      v-for="(sv, idx) in userServers"
      :sv="sv"
      :key="idx"
    />
    
  </div>
</template>

<script>
import ServerListItem from './serverListItem.vue';

export default {
  name: "ServerList",
  props: {
    appState: Array,
    textChannel: String,
    userServers: Array,
    serverID: String
  },
  components: {
    ServerListItem
  }
};
</script>

<style scoped>
.server_list {
  transition: 0.2s; 
  background-color: var(--dark-foreground-element-color);
  position: absolute;
  top: 75px;
  left: 5px;
  height: calc(100vh - 155px);
  width: 60px;
  z-index: 9999;
  border-radius: 40px 40px 40px 40px;
  border: 2px solid transparent;
}

.server_list:hover {
  border-color: var(--dark-foreground-element-color);
  border-radius: 20px 20px 20px 20px;

  transition: border-radius 0.2s;
}
</style>
