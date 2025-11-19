<template>
  <div >
    <img :src="source" />
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
