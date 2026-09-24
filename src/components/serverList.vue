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
  background-color: var(--bg-rail);
  width: 60px;
  flex-grow: 1;
  z-index: 9999;
  border-radius: 40px 40px 40px 40px;
  border: 1px solid var(--border-subtle);
  overflow-x: scroll;
}

.server_list:hover {
  border-color: rgba(196, 138, 146, 0.18);
  border-radius: 20px 20px 20px 20px;

  transition: border-radius 0.2s;
}
</style>
