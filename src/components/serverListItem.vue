<template>
  <img  v-bind:src="sv['img_url']" class='server-icon'/>
</template>

<script>
export default {
  name: "ServerListItem",
  props: {
    sv: Object,
    appState: Array,
    textChannel: String,
    userServers: Array,
    serverID: String
  },
};

</script>


<style scoped>

.server-icon {
  margin-top: 0px;
  margin-top:10px;
  width:  50px;
  height: 50px;
  border-radius: 40px 40px 40px 40px;
  margin-top: 10px;
}

.server-icon:hover {
  border-radius: 25px 25px 25px 25px;
  transition: 0.2s;
}

</style>
