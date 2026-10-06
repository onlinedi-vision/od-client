<template>
    <div style='display:flex'>
      <div class='user-list-icon-container'>
        <AvatarImg
          :src="svuser['img_url']"
          :width="44"
          :height="44"
          img-class="user-list-icon"
        />
      </div>
      <div class='server-user-username'>{{svuser['username']}}</div>
    </div>
</template>

<script>
import AvatarImg from './AvatarImg.vue'

export default {
  name: "ServerUsersListItem",
  components: { AvatarImg },
  props: {
    appState: Array,
    serverID: String,
    svuser: Object
  }
};

</script>

<style scoped>
:deep(.user-list-icon) {
  margin-left: 5px;
  margin-top: 2px;
}

.server-user-username {
  padding-left:15px;
  margin:auto;
  margin-left:0px;
  text-align:center;
}

.user-list-icon-container {
  width: 50px;
  height: 50px;
  padding-left: 3px;
}
</style>
