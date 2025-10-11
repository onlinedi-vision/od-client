<script src="./app.js"/>
<template >
  <main class="container" v-if="done && loggedin">

    <form enctype='multipart/form-data' id="file-form" class="row" @submit.prevent="greet">
      <input id="file-upload" type="file"/>
      <label for="file-upload" class="custom-file-upload">
        <h3 style="margin-top: 10px;"class="fa fa-cloud-upload fa-plus"><b>+</b></h3>
      </label>
      <input  id="greet-input" v-model="name" placeholder="Type a message..." />
      <button id="send" type="submit" @click="send_message(name)">Send</button>
    </form>
    <img src='https://media1.tenor.com/m/viIU4ICp1N8AAAAd/dance.gif' width='60px' height='60px' class='cui' style='margin-bottom:0px;'/>
    <button class="createSButton" @click="createServer()"><h2 style="margin-top: 12px">+</h2></button>
    <div  class = "column">  
      <template v-for="(sv, idx) in uservers">
        <img @click="change_server(sv['serverID'])" v-bind:src="sv['img_url']" width='50px' height='50px' class='user-icon' style='margin-top: 0px;margin-top:10px;'/>
      </template>
    </div>
      
    <div v-if="showSIDvar" class="login" style="display:flex;flex-direction:row; left:100px; top:50px; height:30px; width:660px; z-index:999999;">
      <i style="font-size: 12px"> {{serverID}} </i>
    </div>
       <div class = "chanels">
         <div class ="server-header" @click="showSID()" style="border-bottom: 2px solid #141414; height: 60px;"> <h3 style="margin-bottom: 0px;margin-top:5px"> {{uservers[uservers.findIndex(obj => obj.serverID == serverID)].name}} </h3> 
         </div>
           <template v-if="done">
             <template v-if="appState[appState.findIndex(obj => obj.serverID == serverID)].storedChannels !== undefined">
              <template v-for="(div, index) in appState[appState.findIndex(obj => obj.serverID == serverID)].storedChannels" :key="index">
                <template v-if="div['channelTag'] === textChannel">     
                  <button @click="change_channel(div['channelTag'])" class="channel_button" style="background-color: #a87678; color: white"># {{div['channelTag']}}</button>
                </template>
                <template v-else>
                  <button @click="change_channel(div['channelTag'])" class="channel_button"># {{div['channelTag']}}</button>
                </template>
              </template>
            </template>
           </template>
        <button @click="create_channel(serverID)" class="channel_button" style=" position: relative; text-align: center; padding-left: 0px; padding-right: 0px;"><b>+</b></button>

      </div>
      <div v-if="createChannelPopUp" class="login" style="display:flex;flex-direction:row; left:200px; top:100px; height:50px; width:280px">
        <input id="greet-input" v-model="nchn" style="width:200px" placeholder="new_channel_name..."/>
        <button id="send" type="submit" style="width:80px" @click="createChannel()">Create</button>

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
      </div>


      <div id="chat">
        <template v-if="done">
          <template v-for="(div, index) in appState[appState.findIndex(obj => obj.serverID == serverID)].storedChannels[appState[appState.findIndex(obj => obj.serverID == serverID)].storedChannels.findIndex(obj => obj.channelTag==textChannel)]['messages']" :key="div">
            <div class="comp-mess">
              <img v-if="svusers[svusers.findIndex(obj => obj.username==div['username'])] !== undefined" v-bind:src="svusers[svusers.findIndex(obj => obj.username==div['username'])]['img']" width='40px' height='40px' class='user-icon' style='margin-top: 10px;margin-bottom:10px;'/>
              <img v-else src="https://cdn.discordapp.com/attachments/556118918217859083/1335293396688048320/iu.png?ex=679fa462&is=679e52e2&hm=eb4e496a43cf6e52aad8833e027115767b84d9ec96b3eb9b755e7f3597e3f601&" width='40px' height='40px' class='user-icon' style='margin-top: 10px;margin-bottom:10px;'/>
              <div class="message">
            
                <div class="user" style="padding-top:6px;">
                  <div ><i>{{div['username']}}</i></div>
                  <div class="mdate" style="font-size:12px;padding-left:5px;padding-top:1px;"><i>{{get_date(Number(div['datetime']))}}</i></div>
                </div>
                <div v-if="div['m_content'].startsWith('https:') && ['.jpg','.png','.jpeg','.gif'].some(char => div['m_content'].endsWith(char))">
                  <img v-bind:src="div['m_content']">
                </div>
                <div v-else-if="div['m_content'].startsWith('https:') && ['.ogg','.mp4'].some(char => div['m_content'].endsWith(char))">
                  <video v-bind:src="div['m_content']" type="video/ogg" controls>
                  </video>
                </div>
                <div style="font-size: 17px;" v-else>
                  {{div['m_content']}}
                </div>
              </div>
            </div>
          </template>
        </template
      </div>
    </div>

      <div id="channel-header">
        <h3>{{this.textChannel}}</h3>
      </div>

      <div class = "server-users">
        <template v-for="(svuser,index) in svusers" :key="index">
            <img v-bind:src="svuser['img']" width='50px' class="user-icon"/>
        </template>
      </div>
  </main>
    <main v-else-if="!loggedin">
      <LogInWindow :logInSelected='logInSelected' :lError='lError' :lErrorText='lErrorText' @changeLogIn='changeLogIn' @login='logIn' @signup='signUp' />
    </main>
    
</template>

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

