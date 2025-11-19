<template>
  <div>
    <video
      :src="source"
      type="video/mp4"
      controls
    />
  </div>
</template>
<script>

export default {
  name: "VideoMessage",
  props: {
    source: String,
  }
}

</script>
