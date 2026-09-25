<template>
  <div class="comp-mess" :class="{ 'comp-mess-compact': !showHeader }">
    <div class="mess-body">
      <div class="mess-avatar" :class="{ 'mess-avatar-empty': !showHeader }">
        <AvatarImg
          v-if="showHeader"
          :src="getUser(msg.username)?.img_url"
          :width="avatarSize"
          :height="avatarSize"
          square
          img-class="mess-pfp"
        />
      </div>

      <div class="mess-content">
        <div v-if="showHeader" class="user-meta">
          <b class="user-name">{{ msg.username }}</b>
          <span class="mdate">
            <i>{{ getDate(Number(msg.datetime)) }}</i>
          </span>
        </div>
        <span v-else class="mdate mdate-compact">
          <i>{{ getDate(Number(msg.datetime)) }}</i>
        </span>

        <ImageMessage v-if="isImage(msg.m_content)" :source="msg.m_content" />

        <VideoMessage
          v-else-if="isVideo(msg.m_content)"
          :source="msg.m_content"
        />

        <div v-else-if="mounted" class="mess-text">
          <MarkdownRender
            :content="msg.m_content"
            :render-code-blocks-as-pre="true"
          />
        </div>

        <pre v-else class="mess-text">{{ msg.m_content }}</pre>
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

/** Square side length: tuned so stacked one-line headers sit flush avatar-to-avatar */
const avatarSize = 52

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
  --mess-avatar: 52px;
  padding: 0;
  margin: 0;
  width: 100%;
  box-sizing: border-box;
}

.mess-body {
  display: grid;
  grid-template-columns: var(--mess-avatar) minmax(0, 1fr);
  column-gap: 0;
  align-items: start;
  width: 100%;
}

.mess-avatar {
  width: var(--mess-avatar);
  height: var(--mess-avatar);
  line-height: 0;
  z-index: 1;
}

.mess-avatar:not(.mess-avatar-empty) {
  -webkit-mask-image: linear-gradient(to right, #000 0%, #000 38%, transparent 100%);
  mask-image: linear-gradient(to right, #000 0%, #000 38%, transparent 100%);
}

.mess-avatar-empty {
  visibility: hidden;
}

.mess-avatar :deep(.avatar-img) {
  width: var(--mess-avatar) !important;
  height: var(--mess-avatar) !important;
}

.mess-content {
  min-width: 0;
  padding: 0 0 0 10px;
  margin-left: -8px;
  text-align: left;
  position: relative;
  z-index: 0;
}

.comp-mess-compact .mess-content {
  margin-left: 0;
  padding-left: 0;
}

.comp-mess:not(.comp-mess-compact) .mess-content {
  min-height: var(--mess-avatar);
}

.user-meta {
  display: flex;
  align-items: baseline;
  flex-wrap: wrap;
  gap: 0 6px;
  line-height: 1.15;
  margin: 0 0 2px;
}

.user-name {
  font-size: 17px;
  font-weight: 700;
}

.mdate {
  font-size: 12px;
  color: transparent;
}

.comp-mess:hover .mdate {
  color: var(--text-muted);
}

.mdate-compact {
  display: block;
  height: 0;
  overflow: hidden;
  margin: 0;
  padding: 0;
}

.comp-mess-compact:hover .mdate-compact {
  height: auto;
  margin-bottom: 2px;
}

.mess-text {
  font-size: 17px;
  line-height: 1.25;
  margin: 0;
  text-align: left;
}

.mess-text :deep(p) {
  margin: 0;
}
</style>
