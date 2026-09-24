<template>
  <div class="comp-mess" :class="{ 'comp-mess-compact': !showHeader }">
    <div v-if="showHeader" class="avatar-col">
      <AvatarImg
        :src="getUser(msg.username)?.img_url"
        :width="40"
        :height="40"
        img-class="user-icon"
        :img-style="{ marginTop: '10px', marginBottom: '10px' }"
      />
    </div>
    <div v-else class="avatar-spacer" aria-hidden="true" />

    <div class="message">
      <div v-if="showHeader" class="user" style="padding-top: 6px">
        <div><b style="font-size: 18px">{{ msg.username }}</b></div>
        <div
          class="mdate"
          style="font-size: 12px; padding-left: 5px; padding-top: 1px"
        >
          <i>{{ getDate(Number(msg.datetime)) }}</i>
        </div>
      </div>
      <div
        v-else
        class="mdate mdate-compact"
        style="font-size: 12px; padding-top: 1px"
      >
        <i>{{ getDate(Number(msg.datetime)) }}</i>
      </div>

      <ImageMessage
        v-if="isImage(msg.m_content)"
        :source="msg.m_content"
      />

      <VideoMessage
        v-else-if="isVideo(msg.m_content)"
        :source="msg.m_content"
      />

      <div v-else-if="mounted" style="font-size: 17px">
        <MarkdownRender
          :content="msg.m_content"
          :render-code-blocks-as-pre="true"
        />
      </div>

      <div v-else style="font-size: 17px">
        <pre style="margin: 0px">{{ msg.m_content }}</pre>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'
import { onMounted, ref } from 'vue'
import MarkdownRender from 'vue-renderer-markdown'
import ImageMessage from './messages/ImageMessage.vue'
import VideoMessage from './messages/VideoMessage.vue'
import AvatarImg from './AvatarImg.vue'

const mounted = ref(false)
onMounted(() => {
  mounted.value = true
})

const props = defineProps({
  appState: Array,
  serverID: String,
  textChannel: String,
  get_date: Function,
  msg: Object,
  showHeader: { type: Boolean, default: true },
})

const currentServer = computed(() =>
  props.appState.find((sv) => sv.serverID === props.serverID)
)

function getUser(username) {
  return currentServer.value?.serverUsers?.find(
    (u) => u.username === username
  )
}

function isImage(content) {
  return (
    typeof content === 'string' &&
    content.startsWith('https:') &&
    ['.jpg', '.png', '.jpeg', '.gif'].some((ext) =>
      content.toLowerCase().endsWith(ext)
    )
  )
}

function isVideo(content) {
  return (
    typeof content === 'string' &&
    content.startsWith('https:') &&
    ['.ogg', '.mp4', '.webm'].some((ext) =>
      content.toLowerCase().endsWith(ext)
    )
  )
}

function getDate(ms) {
  return props.get_date
    ? props.get_date(ms)
    : new Date(ms).toLocaleString()
}
</script>

<style scoped>
.comp-mess {
  display: flex;
  align-items: flex-start;
}

.avatar-col {
  flex-shrink: 0;
  width: 50px;
}

.avatar-spacer {
  flex-shrink: 0;
  width: 50px;
}

.message {
  margin-left: 10px;
  min-width: 0;
}

.mdate-compact {
  color: transparent;
  height: 0;
  overflow: hidden;
  margin: 0;
  padding: 0 !important;
}

.comp-mess-compact:hover .mdate-compact {
  color: var(--text-muted);
  height: auto;
  margin-bottom: 2px;
}
</style>
