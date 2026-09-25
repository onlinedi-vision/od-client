<template>
  <LoadingScreen v-if="!done" />
  <main
    class="container"
    :class="{ 'container--mobi': isMobile }"
    v-if="done && loggedin"
  >
    <div
      v-if="isMobile"
      class="mobi-shell"
      @touchstart.passive="onMobiTouchStart"
      @touchmove="onMobiTouchMove"
      @touchend="onMobiTouchEnd"
      @touchcancel="onMobiTouchEnd"
    >
      <div class="mobi-track" :style="mobiTrackStyle">
        <div class="mobi-panel mobi-panel--menu">
          <div class="container-v">
            <AvatarImg
              @click="openSettings()"
              :src="myPfp"
              :width="60"
              :height="60"
              img-class="cui"
              :img-style="{ marginBottom: '0px' }"
            />
            <ServerList
              :userServers="userServers"
              :appState="appState"
              :serverID="serverID"
              :textChannel="textChannel"
              @changeServer="change_server"
            />
            <button class="createSButton" @click="createServer()">
              <h2 style="margin-top: 12px">+</h2>
            </button>
          </div>
          <ChannelList
            :appState="appState"
            :serverID="serverID"
            :textChannel="textChannel"
            :done="done"
            :userServers="userServers"
            @showSID="showSID"
            @changeChannel="onMobileChannelSelect"
            @createChannel="create_channel"
            @deleteChannel="deleteChannel"
          />
          <div class="mobi-users">
            <ServerUsersList :appState="appState" :serverID="serverID" />
          </div>
        </div>
        <div class="mobi-panel mobi-panel--chat">
          <div class="container-v chat-column">
            <div id="channel-header">
              <h3>{{ textChannel }}</h3>
            </div>
            <ChatWindow
              :appState="appState"
              :serverID="serverID"
              :textChannel="textChannel"
              :get_date="get_date"
            />
            <div>
              <form
                enctype="multipart/form-data"
                id="file-form"
                class="row composer-bar"
                @submit.prevent="greet"
              >
                <input id="file-upload" type="file" @change="onFileChange" />
                <div v-if="selectedFileUrl" class="file-preview">
                  <div style="font-size: 12px; color: var(--text-muted)">
                    {{ selectedFile?.name }} ({{ Math.round(selectedFile.size / 1024) }} KB)
                  </div>
                  <div v-if="selectedFile && selectedFile.type.startsWith('image/')">
                    <img
                      :src="selectedFileUrl"
                      alt="preview"
                      style="max-width: 150px; border-radius: 6px"
                    />
                  </div>
                  <div v-else-if="selectedFile && selectedFile.type.startsWith('video/')">
                    <video
                      :src="selectedFileUrl"
                      controls
                      style="max-width: 150px; border-radius: 6px"
                    ></video>
                  </div>
                  <div v-else style="max-width: 150px; align-items: center">
                    <i>¯\_(ツ)_/¯</i>
                  </div>
                  <button type="button" @click="clearSelectedFile()" style="margin-top: 6px">
                    Remove
                  </button>
                </div>
                <label for="file-upload" class="custom-file-upload">
                  <h3 style="margin-top: 10px" class="fa fa-cloud-upload fa-plus"><b>+</b></h3>
                </label>
                <textarea
                  maxlength="2000"
                  rows="1"
                  style="font-size: 16px; field-sizing: content"
                  id="greet-input"
                  v-model="message"
                  placeholder="Type a message..."
                  oninput="this.style.height = 'auto'; this.style.height = (this.scrollHeight - 10) + 'px';"
                />
                <button id="send" type="submit" @click="send_message(message)">Send</button>
              </form>
            </div>
          </div>
        </div>
      </div>
    </div>

    <template v-else>
      <div class="container-v">
        <AvatarImg
          @click="openSettings()"
          :src="myPfp"
          :width="60"
          :height="60"
          img-class="cui"
          :img-style="{ marginBottom: '0px' }"
        />
        <ServerList
          :userServers="userServers"
          :appState="appState"
          :serverID="serverID"
          :textChannel="textChannel"
          @changeServer="change_server"
        />
        <button class="createSButton" @click="createServer()">
          <h2 style="margin-top: 12px">+</h2>
        </button>
      </div>
      <ChannelList
        :appState="appState"
        :serverID="serverID"
        :textChannel="textChannel"
        :done="done"
        :userServers="userServers"
        @showSID="showSID"
        @changeChannel="change_channel"
        @createChannel="create_channel"
        @deleteChannel="deleteChannel"
      />
      <div class="container-v chat-column">
        <div id="channel-header">
          <h3>{{ textChannel }}</h3>
        </div>
        <ChatWindow
          :appState="appState"
          :serverID="serverID"
          :textChannel="textChannel"
          :get_date="get_date"
        />
        <div>
          <form
            enctype="multipart/form-data"
            id="file-form"
            class="row composer-bar"
            @submit.prevent="greet"
          >
            <input id="file-upload" type="file" @change="onFileChange" />
            <div v-if="selectedFileUrl" class="file-preview">
              <div style="font-size: 12px; color: var(--text-muted)">
                {{ selectedFile?.name }} ({{ Math.round(selectedFile.size / 1024) }} KB)
              </div>
              <div v-if="selectedFile && selectedFile.type.startsWith('image/')">
                <img
                  :src="selectedFileUrl"
                  alt="preview"
                  style="max-width: 150px; border-radius: 6px"
                />
              </div>
              <div v-else-if="selectedFile && selectedFile.type.startsWith('video/')">
                <video
                  :src="selectedFileUrl"
                  controls
                  style="max-width: 150px; border-radius: 6px"
                ></video>
              </div>
              <div v-else style="max-width: 150px; align-items: center">
                <i>¯\_(ツ)_/¯</i>
              </div>
              <button type="button" @click="clearSelectedFile()" style="margin-top: 6px">
                Remove
              </button>
            </div>
            <label for="file-upload" class="custom-file-upload">
              <h3 style="margin-top: 10px" class="fa fa-cloud-upload fa-plus"><b>+</b></h3>
            </label>
            <textarea
              maxlength="2000"
              rows="1"
              style="font-size: 16px; field-sizing: content"
              id="greet-input"
              v-model="message"
              placeholder="Type a message..."
              oninput="this.style.height = 'auto'; this.style.height = (this.scrollHeight - 10) + 'px';"
            />
            <button id="send" type="submit" @click="send_message(message)">Send</button>
          </form>
        </div>
      </div>
      <div style="margin-left: auto">
        <ServerUsersList :appState="appState" :serverID="serverID" />
      </div>
    </template>

    <div
      v-if="showSIDvar"
      class="login"
      style="
        display: flex;
        flex-direction: row;
        left: 100px;
        top: 50px;
        height: 30px;
        width: 660px;
        z-index: 999999;
      "
    >
      <i style="font-size: 12px"> {{ serverID }} </i>
    </div>

    <div
      v-if="createChannelPopUp"
      class="login"
      style="
        display: flex;
        flex-direction: row;
        left: 200px;
        top: 100px;
        height: 50px;
        width: 280px;
      "
    >
      <input id="greet-input" v-model="nchn" style="width: 200px" placeholder="new_channel_name..." />
      <button id="send" type="submit" style="width: 80px" @click="createChannel()">Create</button>
      <button id="send" type="submit" style="width: 80px" @click="create_channel_cancel()">
        Cancel
      </button>
    </div>

    <div v-if="createServerPopUp" class="login">
      <h3>Create Server</h3>
      <input class="csv" v-model="newSvName" placeholder="Server Name..." />
      <input class="csv" v-model="newImgUrl" placeholder="Image URL..." />
      <input class="csv" v-model="newShDesc" placeholder="Short Description..." />
      <button class="csvb" @click="createSever()">Create Server</button>

      <h3 style="padding-top: 30px">Join Server</h3>
      <input class="csv" v-model="joinserverID" placeholder="Server ID..." />
      <button class="csvb" @click="joinServer()">Join Server</button>
      <button class="csvb" @click="createServerCancel()">Cancel</button>
    </div>
  </main>
  <main v-else-if="done && !loggedin">
    <LogInWindow
      :logInSelected="logInSelected"
      :lError="lError"
      :lErrorText="lErrorText"
      @changeLogIn="changeLogIn"
      @login="logIn"
      @signup="signUp"
    />
  </main>
  <div @click="closeSettings()" class="settings-background" v-if="settingsOpen" />
  <SettingsWindow
    :userName="username"
    :profilePic="myPfp"
    v-if="settingsOpen"
    @closeSettings="closeSettings"
    @logOut="logOut"
    @setOwnPfp="setOwnPfp"
  />
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, watch, getCurrentInstance } from 'vue';

const MOBILE_BREAKPOINT = 768;
const SWIPE_THRESHOLD = 56;

const width = ref(window.innerWidth);
function onResize() {
  width.value = window.innerWidth;
}

const isMobile = computed(() => width.value < MOBILE_BREAKPOINT);
/** false = chat panel, true = servers/channels/users panel */
const viewChannels = ref(false);

const mobiDragging = ref(false);
const mobiDragX = ref(0);
let mobiStartX = 0;
let mobiStartY = 0;
let mobiTouchAxis = null;

const mobiTrackStyle = computed(() => {
  if (!isMobile.value) return {};
  const panelW = width.value;
  const base = viewChannels.value ? 0 : -panelW;
  const x = base + mobiDragX.value;
  return {
    transform: `translate3d(${x}px, 0, 0)`,
    transition: mobiDragging.value ? 'none' : 'transform 0.28s ease-out',
  };
});

function onMobiTouchStart(e) {
  if (!isMobile.value || e.touches.length !== 1) return;
  mobiStartX = e.touches[0].clientX;
  mobiStartY = e.touches[0].clientY;
  mobiTouchAxis = null;
  mobiDragging.value = true;
  mobiDragX.value = 0;
}

function onMobiTouchMove(e) {
  if (!mobiDragging.value || e.touches.length !== 1) return;
  const dx = e.touches[0].clientX - mobiStartX;
  const dy = e.touches[0].clientY - mobiStartY;
  if (!mobiTouchAxis) {
    if (Math.abs(dx) < 8 && Math.abs(dy) < 8) return;
    mobiTouchAxis = Math.abs(dx) > Math.abs(dy) ? 'x' : 'y';
  }
  if (mobiTouchAxis !== 'x') return;
  e.preventDefault();
  const panelW = width.value;
  if (viewChannels.value) {
    mobiDragX.value = Math.max(-panelW, Math.min(0, dx));
  } else {
    mobiDragX.value = Math.max(0, Math.min(panelW, dx));
  }
}

function onMobiTouchEnd() {
  if (!mobiDragging.value) return;
  mobiDragging.value = false;
  if (mobiTouchAxis === 'x') {
    if (!viewChannels.value && mobiDragX.value > SWIPE_THRESHOLD) {
      viewChannels.value = true;
    } else if (viewChannels.value && mobiDragX.value < -SWIPE_THRESHOLD) {
      viewChannels.value = false;
    }
  }
  mobiDragX.value = 0;
  mobiTouchAxis = null;
}

const inst = getCurrentInstance();
function onMobileChannelSelect(tag) {
  inst?.proxy?.change_channel(tag);
  viewChannels.value = false;
}

watch(isMobile, (mobile) => {
  if (!mobile) viewChannels.value = false;
});

onMounted(() => {
  window.addEventListener('resize', onResize);
});
onUnmounted(() => {
  window.removeEventListener('resize', onResize);
});
</script>

<script>
import app from "./app.js"
import ChatWindow from "./components/chatWindow.vue"
import LogInWindow from "./components/login.vue"
import SettingsWindow from './components/settings.vue'
import ChannelList from "./components/channelList.vue";
import ServerList from "./components/serverList.vue";
import ServerUsersList from "./components/serverUsersList.vue";
import LoadingScreen from "./components/LoadingScreen.vue";
import AvatarImg from "./components/AvatarImg.vue";

export default {
  ...app,
  name: "App",
  components: {
    LogInWindow,
    ChatWindow,
  	SettingsWindow,
    ChannelList,
    ServerList,
    ServerUsersList,
    LoadingScreen,
    AvatarImg
  }
}
</script>

<style scoped>
.logo.vite:hover {
  filter: drop-shadow(0 0 2em #747bff);
}

.logo.vue:hover {
  filter: drop-shadow(0 0 2em #249b73);
}

#app {
  height: 100%;

}
</style>

<style src="./style.css"/>
