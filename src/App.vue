<template >
  <main class="container" v-if="done && loggedin">

    <form enctype='multipart/form-data' id="file-form" class="row" @submit.prevent="greet">
      <input id="file-upload" type="file" @change="onFileChange" />
	  <!--   codul pentru preview la incarcare document -->
      <div v-if="selectedFileUrl" class="file-preview">
        <div style="font-size:12px;color:#aaa;">
          {{ selectedFile?.name }} ({{ Math.round(selectedFile.size / 1024) }} KB)
        </div>
        <div v-if="selectedFile && selectedFile.type.startsWith('image/')">
          <img :src="selectedFileUrl" alt="preview" style="max-width:150px; border-radius:6px;" />
        </div>
        <div v-else-if="selectedFile && selectedFile.type.startsWith('video/')">
          <video :src="selectedFileUrl" controls style="max-width:150px; border-radius:6px;"></video>
        </div>
        <div v-else style="max-width:150px; align-items: center;">
          <i>¯\_(ツ)_/¯</i>
        </div>
        <button type="button" @click="clearSelectedFile()" style=" margin-top:6px;">Remove</button>
      </div>
      <label for="file-upload" class="custom-file-upload">
        <h3 style="margin-top: 10px;"class="fa fa-cloud-upload fa-plus"><b>+</b></h3>
      </label>
      <input  id="greet-input" v-model="name" placeholder="Type a message..." />
      <button id="send" type="submit" @click="send_message(name)">Send</button>
    </form>
    <img @click="openSettings()" v-bind:src='myPfp' width='60px' height='60px' class='cui'
      style='margin-bottom:0px;' />
    <button class="createSButton" @click="createServer()">
      <h2 style="margin-top: 12px">+</h2>
    </button>
    <div class="column">
      <template v-for="(sv, idx) in userServers">
        <img @click="change_server(sv['serverID'])" v-bind:src="sv['img_url']" width='50px' height='50px' class='user-icon' style='margin-top: 0px;margin-top:10px;'/>
      </template>
    </div>
      
    <div v-if="showSIDvar" class="login" style="display:flex;flex-direction:row; left:100px; top:50px; height:30px; width:660px; z-index:999999;">
      <i style="font-size: 12px"> {{serverID}} </i>
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
      />

      <div v-if="createChannelPopUp" class="login" style="display:flex;flex-direction:row; left:200px; top:100px; height:50px; width:280px">
        <input id="greet-input" v-model="nchn" style="width:200px" placeholder="new_channel_name..."/>
        <button id="send" type="submit" style="width:80px" @click="createChannel()">Create</button>
        <button id="send" type="submit"  style="width:80px;" @click="create_channel_cancel()">Cancel</button>

      </div>



      <div v-if="createServerPopUp" class="login">
        <h3>Create Server</h3>
        <input class="csv" v-model="newSvName" placeholder="Server Name..." />
        <input class="csv" v-model="newImgUrl" placeholder="Image URL..." />
        <input class="csv" v-model="newShDesc" placeholder="Short Description...">
        <button class="csvb" @click="createSeverForReal()">Create Server</button>
      
        <h3 style="padding-top:30px" >Join Server</h3>
        <input class="csv" v-model="joinserverID" placeholder="Server ID..."/>
        <button class="csvb" @click="joinServer()">Join Server</button>
        <button class="csvb" @click="createServerCancel()">Cancel</button>
      </div>

      <ChatWindow
        :appState="appState"
        :serverID="serverID"
        :textChannel="textChannel"
        :get_date="get_date"
      />

      <div id="channel-header">
        <h3>{{this.textChannel}}</h3>
      </div>

      <div class = "server-users">
        <template v-for="(svuser,index) in appState[appState.findIndex(obj => obj.serverID == serverID)].serverUsers" :key="index">
            <template style='display:flex'>
              <div width='50px' height='50px'>
                <img v-if="svuser['img_url'] != ''" v-bind:src="svuser['img_url']" width='50px' height='50px' class="user-list-icon"/>
                <img v-else v-bind:src="'a'" width='50px' height='50px' class="user-list-icon"/>
              </div>
              <div style='padding-left:10px;margin:auto;margin-left:0px;text-align:center'>{{svuser['username']}}</div>
            </template>
        </template>
      </div>
  </main>
    <main v-else-if="!loggedin">
      <LogInWindow :logInSelected='logInSelected' :lError='lError' :lErrorText='lErrorText' @changeLogIn='changeLogIn' @login='logIn' @signup='signUp' />
    </main>
	<div @click="closeSettings()" class="settings-background" v-if="settingsOpen" />
	<SettingsWindow :userName="username" :profilePic="myPfp" v-if="settingsOpen" @closeSettings="closeSettings" @logOut="logOut" @setOwnPfp="setOwnPfp" />
</template>

<script>
import app from "./app.js"
import ChatWindow from "./components/chatWindow.vue"
import LogInWindow from "./components/login.vue"
import SettingsWindow from './components/settings.vue'
import ChannelList from "./components/channelList.vue";

export default {
  ...app,
  name: "App",
  components: {
    LogInWindow,
    ChatWindow,
	SettingsWindow,
  ChannelList
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
