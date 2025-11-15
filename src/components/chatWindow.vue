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

</script>
