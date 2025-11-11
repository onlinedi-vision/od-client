<template>
  <div id="chat">
    <template
      v-if="
        currentServer?.storedChannels &&
        currentChannel?.messages &&
        currentServer?.serverUsers
      "
    >
      <template
        v-for="(msg, index) in currentChannel.messages"
        :key="index"
      >
        <div class="comp-mess" height="100px">
          <img
            v-if="getUser(msg.username)?.img_url"
            :src="getUser(msg.username).img_url"
            width="40"
            height="40"
            class="user-icon"
            style="margin-top: 10px; margin-bottom: 10px;"
          />
          <img
            v-else
            src="/placeholder.png"
            width="40"
            height="40"
            class="user-icon"
            style="margin-top: 10px; margin-bottom: 10px;"
          />

          <div class="message">
            <div class="user" style="padding-top:6px;">
              <div><i>{{ msg.username }}</i></div>
              <div
                class="mdate"
                style="font-size:12px; padding-left:5px; padding-top:1px;"
              >
                <i>{{ getDate(Number(msg.datetime)) }}</i>
              </div>
            </div>

            <!-- Image messages -->
            <div v-if="isImage(msg.m_content)">
              <img :src="msg.m_content" />
            </div>

            <!-- Video messages -->
            <div v-else-if="isVideo(msg.m_content)">
              <video
                :src="msg.m_content"
                type="video/mp4"
                controls
              />
            </div>

            <!-- Text messages -->
            <div v-else-if='mounted' style="font-size: 17px;"><client-side><MarkdownRender :content="msg.m_content" :render-code-blocks-as-pre="true"/></client-side></div>

            <div v-else style="font-size: 17px;"><pre style='margin:0px'>{{ msg.m_content }}</pre></div>
          </div>
        </div>
      </template>
    </template>
  </div>
</template>

<script setup>
import { computed } from "vue"
import { onMounted, ref } from 'vue'
import MarkdownRender from 'vue-renderer-markdown'
import { MarkdownCodeBlockNode, setCustomComponents } from 'vue-renderer-markdown'
/* Currently disabled due to bugs and performance issues
setCustomComponents({
  code_block:MarkdownCodeBlockNode
})
*/
const mounted = ref(false)
onMounted(() => {
  mounted.value = true
})

const props = defineProps({
  appState: Array,
  serverID: String,
  textChannel: String,
  get_date: Function
})

const currentServer = computed(() =>
  props.appState.find((sv) => sv.serverID === props.serverID)
)

const currentChannel = computed(() =>
  currentServer.value?.storedChannels?.find(
    (ch) => ch.channelTag === props.textChannel
  )
)

function getUser(username) {
  return currentServer.value?.serverUsers?.find(
    (u) => u.username === username
  )
}

function isImage(content) {
  return (
    typeof content === "string" &&
    content.startsWith("https:") &&
    [".jpg", ".png", ".jpeg", ".gif"].some((ext) =>
      content.toLowerCase().endsWith(ext)
    )
  )
}

function isVideo(content) {
  return (
    typeof content === "string" &&
    content.startsWith("https:") &&
    [".ogg", ".mp4", ".webm"].some((ext) =>
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

.message {
  margin-left: 10px;
}
</style>
