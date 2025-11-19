<template>
  <div>
    <img :src="source" class="image" style="border-radius:0px"/>
  </div>
</template>

<script>

export default {
  name: "ImageMessage",
  props: {
    source: String,
  }
}

</script>

<style scoped>
.image {
  border-radius: 0px;
  border: 0px;
}
</style>
