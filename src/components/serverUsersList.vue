<template>
  <div class = "server-users">

    <ServerUsersListItem
    :appState="appState"
    :serverID="serverID"

    v-for="(svuser,index) in appState[appState.findIndex(obj => obj.serverID == serverID)].serverUsers"
    :svuser="svuser"
    :key="index"
    />
  </div>
</template>

<script>
import ServerUsersListItem from './serverUsersListItem.vue';

export default {
  name: "ServerUsersList",
  props: {
    appState: Array,
    serverID: String
  },
  components: {
    ServerUsersListItem
  }
};

</script>

<style scoped>
.server-users {
  overflow-y: hidden;
  overflow-x: scroll;
  transition: 0.2s;
  background-color: var(--dark-foreground-element-color);
  position: absolute;
  top: 5px;
  right: 5px;
  height: calc(100vh - 11px);
  width: 60px;
  border-radius: 40px 40px 40px 40px;
  border: 2px solid transparent;
  z-index: 9999;
}
.server-users:hover {
  width: 200px;
  border-color: var(--dark-foreground-element-color);
  border-radius: 20px 20px 20px 20px;
  transition: 0.2s;
}
</style>
