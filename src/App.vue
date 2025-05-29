<script>
import { invoke } from '@tauri-apps/api/core';
import UserMessage from './UserMessage.vue';
export default {
  data() {
    let token="8d0e40b84a092bf4c5c863d8e64e7eb1281ac9aa9fc66d39363525795f9f30cb";
    let server="1313";
    function compareDate(a, b) {
      if(Number(a.datetime) < Number(b.datetime)) {
        return -1;
      }
      if(Number(a.datetime) > Number(b.datetime)) {
        return 1;
      }
      return 0;

    }
    invoke('getChannels', {token: token, server: server, username: "ana"})
      .then((res) => {
        let channels = JSON.parse(res)['c_list'];
        for(let i = 0; i<channels.length; i++) {
        invoke('getMessages', {token: token, server:server, channel:channels[i]['channel_name'], username: "ana"})
          .then((res) => {
            this.divs.push({'channelTag': channels[i]['channel_name'], 'messages': JSON.parse(res)['m_list'].sort(compareDate)})
          })
          .catch((err) => {
            console.log('err');
          });

          this.divs.push
        }
        this.done=true;
      })
      .catch((err) => {
        console.log('err channels');
      }); 

        return {
          done:false,
          token:token,
          sid: "1313",
          server:true,
          divs: [ {
              'channelTag':'info',
            }
          ],

      
          svusers: [{'img':'https://media1.tenor.com/m/viIU4ICp1N8AAAAd/dance.gif', 'username':'ana'}, {'img':'https://cdn.discordapp.com/attachments/1314144119010103319/1316541431518724096/IMG_4470.webp?ex=679f5181&is=679e0001&hm=b712b31c07433ade4cda2adb16863993be1c6421b16b24606b57e8267d3a239d&', 'username':'rub'}, {'img': 'https://cdn.discordapp.com/attachments/556118918217859083/1335293396688048320/iu.png?ex=679fa462&is=679e52e2&hm=eb4e496a43cf6e52aad8833e027115767b84d9ec96b3eb9b755e7f3597e3f601&', 'username':'doru911'}],
          name: '',
          textChannel: 'info',
          username: 'ana',
    };
  },
  methods: {
    addDiv(message) {
      console.log(message);

      if(message==='')return;
      let sname = this.name;
      this.name = '';
      invoke('sendMessage', {token:this.token, channel: this.textChannel, server: this.sid,  m_content: message, username:'ana' }).then((res) => {
        console.log('!!!!' + this.divs.findIndex(obj => obj.channelTag===this.textChannel));
        this.divs[this.divs.findIndex(obj => obj.channelTag===this.textChannel)]['messages'].push({"username":this.username, "m_content":res});
      }).catch((err) =>{
      this.divs[this.divs.findIndex(obj => obj.channelTag===this.textChannel)]['messages'].push({"username":this.username, "m_content":err});
      });
    },
    get_messages(channel, server, token) {
      invoke('getMessages', {token: token, server:server, channel:channel})
        .then((res) => {
          console.log(res);
        })
        .catch((err) => {
          console.log('err');
        });
    },
    change_channel(newChan) {
      this.textChannel = newChan;
    },
    get_date(timestamp) {
      let date = new Date(Number(timestamp));
      let month = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
      console.log(date);
      return month[Number(date.getMonth())] + ' ' + date.getDate() + ' '+ date.getFullYear() + ' ' + date.getHours() + ':' + date.getMinutes();
    }
  },
  
};

</script>
<template >
  <main class="container" v-if="server">

    <form class="row" @submit.prevent="greet">
      <input id="greet-input" v-model="name" placeholder="Type a message..." />
      <button id="send" type="submit" @click="addDiv(name)">Send</button>
    </form>
    <img src='https://media1.tenor.com/m/viIU4ICp1N8AAAAd/dance.gif' width='60px' height='60px' class='cui' style='margin-bottom:0px;'/>
<img src='https://media1.tenor.com/m/viIU4ICp1N8AAAAd/dance.gif' width='60px' height='60px' class='cui' style='top:75px;margin-bottom:0px;'/>
    <div class = "column">  
    <img src="https://cdn.discordapp.com/attachments/556118918217859083/1335390775579770910/cell-max-realized-he-aint-built-for-this-v0-36k6p9ymw7vb1.jpg?ex=679fff13&is=679ead93&hm=2b49f1eaf204073c73a98ec5debb1afe8fcaf135d93cf3e7fe17e7691514f19d&" width='50px' height='50px' class='user-icon' style='margin-bottom:0px;'/>
    <img src="https://cdn.discordapp.com/attachments/1001236756814897314/1335392112174301224/iu.png?ex=67a00051&is=679eaed1&hm=35ddd407f8649e5d8a244508e3d9dca263b02db836f085878f02d25648b57979&" width='50px' height='50px' class='user-icon' style='margin-top: 0px;margin-bottom:10px;'/>

    </div>
    

       <div class = "chanels">
      <div style="border-bottom: 2px solid #1c0606; height: 60px;"> <h3> Server Name </h3> </div>
         <template v-if="done">

      <template v-for="(div, index) in divs" :key="index">
        <button @click="change_channel(div['channelTag'])" class="channel_button"># {{div['channelTag']}}</button>
      </template>
         </template>
      <button class="channel_button"> voice-channel</button>
    </div>

    <div id="chat">

      <template v-if="done">
      <template v-for="(div, index) in divs[divs.findIndex(obj => obj.channelTag==textChannel)]['messages']" :key="index">
        <div class="comp-mess">
        <img v-bind:src="svusers[svusers.findIndex(obj => obj.username==div['username'])]['img']" width='40px' height='40px' class='user-icon' style='margin-top: 10px;margin-bottom:10px;'/>
        <div class="message">
          
          <div class="user">
            <div><RouterLink to='/profile'>@{{div['username']}}</RouterLink></div>
            <div style="font-size:10px;padding-left:5px;">{{get_date(Number(div['datetime']))}}</div>
          </div>
          <div v-if="div['m_content'].startsWith('https:')">
            <img v-bind:src="div['m_content']">
          </div>
          <div v-else>
            {{div['m_content']}}
          </div>
        </div>
        </div>
      </template>
      </template
        </div>
    </div>
    <div id="channel-header">
    </div>

    <div class = "server-users">
      <template v-for="(svuser,index) in svusers" :key="index">
          <img v-bind:src="svuser['img']" width='50px' class="user-icon"/>
      </template>
    </div>
  </main>
  <main v-else>
    
  </main>
  
</template>


<style scoped>
.logo.vite:hover {
  filter: drop-shadow(0 0 2em #747bff);
}

.logo.vue:hover {
  filter: drop-shadow(0 0 2em #249b73);
}

</style>
<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;
    
  color: #efa7aa;
  background-color: #3d2222;/*#422828;*/



  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin: 0;
  height: 100%;
  /*padding-top: 10vh;*/
  /*display: flex;
  --flex-direction: column;*/
  justify-content: center;
  text-align: center;
}

.logo {
  height: 6em;
  padding: 1.5em;
  will-change: filter;
  transition: 0.75s;
}

.user-icon {
  border-radius: 40px 40px 40px 40px;
  margin-top: 10px;
}

.cui {
  position: absolute;
  top: 5px;
  left: 5px;
  border-radius: 40px 40px 40px 40px;
  border: 2px solid transparent;
transition: 0.2s;
}

.logo.tauri:hover {
  filter: drop-shadow(0 0 2em #24c8db);
}

.row {
  display: flex;
  justify-content: center;
  position: absolute;
  bottom: 5px;
  right: 0.5vw;
  left: 260px;
  width: calc(100vw - 267px);
  z-index: 9999;
}

.channel_button {
  background-color: #331515;
}

.column {
    transition: 0.2s; 
    background-color: #1c0606;
    position: absolute;
    top: 145px;
    left: 5px;
    height: calc(100vh - 155px);
    width: 60px;
    z-index: 9999;
    border-radius: 40px 40px 40px 40px;
    border: 2px solid transparent;
  }

  .server-users {
    transition: 0.2s;
    background-color: #1c0606;
    position: absolute;
    top: 5px;
    right: 5px;
    height: calc(100vh - 60px);
    width: 60px;
    border-radius: 40px 40px 40px 40px;
    border: 2px solid transparent;
  }

  .chanels {

    background-color: #331515;
    position: absolute;
    top: 5px;
    left: 73px;
    height: calc(100vh - 15px);
    width: 180px;
    border: 2px  solid transparent;
    display: flex;
    z-index: 9999;
    flex-direction: column;
    overflow-y: scroll;
  }


  .message {
    text-align: left;
    padding-left: 10px;
  }

  .comp-mess {
    text-align: left;
    padding-left: 10px;
    display: flex;
  }


  .user {
    display:flex;
  }

  #chat {
    display: flex;
    flex-direction: column;
    position: absolute;
    left: 260px;
    width: calc(100vw - 332px);
    top: 68px;
    height: calc(100vh - 120px);
    overflow-y: scroll;
  }

  #channel-header {
    justify-content: center;
    text-align: left;
    position: absolute;
    top: 5px;
    border-bottom: 2px solid #1c0606;
    width: calc(100vw - 332px);
    height: 63px;
    left: 260px;
    background-color: #331515;
    box-shadow: 20px 20px 20px rgba(0, 0, 0, 0.2);
}

::-webkit-scrollbar {
display: none;
}

body {
  height: 100vh;
  margin: 0px;
}

button {
 background-color: #331515;
 color: #efa7aa;
 outline: none;
 border: 0px;
  text-align: left;
  font-size: 16px;
}

button:hover {
  background-color:#efa7aa;
  color: white;
}

.comp-mess:hover {
  background-color: #331515;
}

a {
  font-weight: 500;
  color: #646cff;
  text-decoration: inherit;
}

a:hover {
  color: #535bf2;
}

h1 {
  text-align: center;
}



input {
  margin-right: 0px;
  border: 2px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  /*box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);*/

}
#send {
  padding-left: 0px;
  border-radius: 0px 10px 10px 0px;
  border: 2px solid transparent;
  padding: 10px 10px;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
  width: 62px;
}

#send {
  cursor: pointer;
  background-color: #230909;
  color: #efa7aa;
}

#greet-input:hover {
  border-color: #efa7aa;
  background-color:#351818; 
}

.column:hover {
  border-color: #1c0606;
  border-radius: 20px 20px 20px 20px;

  transition: border-radius 0.2s;
}

.server-users:hover {
  border-color: #1c0606;
  border-radius: 20px 20px 20px 20px;

  transition: border-radius 0.2s;
}

.user-icon:hover {
  border-radius: 25px 25px 25px 25px;
  transition: 0.2s;
}

.cui:hover {
  border-radius: 25px 25px 25px 25px;
  transition: 0.2s;
}

.chanels:hover {
border-color: #efa7aa;
}

#send:hover {
  border-color: #efa7aa;
  background-color:#351818; 
}


input,
button {
  outline: none;
}

#app {
  height: 100%;
}

#greet-input {
  margin-right: 0px;
  background-color: #331515;/*#230d0e;*/ 
  color: #efa7aa;
  position: -webkit-sticky;
  position: sticky;
  bottom: 0;
  width: 100vw;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  a:hover {
    color: #24c8db;
  }

  input,
  button {
    color: #ffffff;
    background-color: #210c0d;
  }
  button:active {
    background-color: #230d0e;
  }
}

</style>
