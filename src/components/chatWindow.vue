<template>
  <div id="chat">
    <template
      v-if="
        currentServer?.storedChannels &&
        currentChannel?.messages &&
        currentServer?.serverUsers
      "
    >
      <MessageItem
        :appState="appState"
        :serverID="serverID"
        :textChannel="textChannel"
        :get_date="get_date"
        :show-header="showHeader(index)"
        v-for="(msg, index) in currentChannel.messages"
        :msg="msg"
        :key="index"
      />
      
    </template>
  </div>
</template>

<script setup>
import { computed } from "vue"
import { onMounted, ref } from 'vue'
import MessageItem from './messageItem.vue';

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

/** Oldest messages sit at higher indices; hide header when same user posted right above. */
function showHeader(index) {
  const messages = currentChannel.value?.messages
  if (!messages?.length) return true
  if (index >= messages.length - 1) return true
  return messages[index + 1].username !== messages[index].username
}

</script>
