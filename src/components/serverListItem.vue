<template>
  <AvatarImg :src="sv['img_url']" :width="50" :height="50" img-class="server-icon" />
</template>

<script>
import AvatarImg from './AvatarImg.vue'

export default {
  name: "ServerListItem",
  components: { AvatarImg },
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

:deep(.server-icon) {
  margin-top: 10px;
}

:deep(.server-icon:hover) {
  border-radius: 25px;
  transition: 0.2s;
}

</style>
