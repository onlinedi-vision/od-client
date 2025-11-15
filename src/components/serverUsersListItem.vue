<template>
    <div style='display:flex'>
      <div width='50px' height='50px'>
        <img v-if="svuser['img_url'] != ''" v-bind:src="svuser['img_url']" width='50px' height='50px' class="user-list-icon"/>
        <img v-else v-bind:src="'a'" width='50px' height='50px' class="user-list-icon"/>
      </div>
      <div style='padding-left:10px;margin:auto;margin-left:0px;text-align:center'>{{svuser['username']}}</div>
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
  width: 44px;
  height: 44px;
  border-radius: 40px 40px 40px 40px;
  margin-top: 2px;
}
</style>

