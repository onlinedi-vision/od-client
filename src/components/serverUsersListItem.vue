<template>
    <div style='display:flex'>
      <div class='user-list-icon-container'>
        <img v-if="svuser['img_url'] != ''" v-bind:src="svuser['img_url']" class="user-list-icon"/>
        <img v-else v-bind:src="'a'" class="user-list-icon"/>
      </div>
      <div class='server-user-username'>{{svuser['username']}}</div>
    </div>
</template>

<script>

export default {
  name: "ServerUsersListItem",
  props: {
    appState: Array,
    serverID: String,
    svuser: Object
  }
};

</script>

<style scoped>
.user-list-icon {
  margin-left: 5px;
  width:  44px;
  height: 44px;
  border-radius: 40px 40px 40px 40px;
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
}
</style>

