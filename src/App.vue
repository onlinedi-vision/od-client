<script>
import { invoke } from '@tauri-apps/api/core';
import UserMessage from './UserMessage.vue';

export default {
  data() {
    let token="16ec6209e46700a7f29fea7b792b53b8f61d3705092bacf4eb853d9f497161b0";
   
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
    
     invoke('getLocalToken')
      .then((res) => {
        token = JSON.parse(res)['token'];
        this.username = JSON.parse(res)['username'];
        console.log(this.username);
        console.log(res);
        invoke('getServers', {token: token, username: this.username})
          .then((res) => {
            console.log('LOL' + token);
            let servers = JSON.parse(res);
            this.token = servers.token;
            token = servers.token;
            
            invoke('writeCredentials', {creds: JSON.stringify({'username': this.username, 'token': token }) });

            for(let i = 0; i < servers['s_list'].length; i++) {
              invoke('getServerInfo', {sid: servers['s_list'][i]})
                .then((si) => {
                  let sinfo = JSON.parse(si);
                  this.uservers.push({'sid': servers['s_list'][i], 'name': sinfo['name'], 'desc': sinfo['desc'], 'img_url': sinfo['img_url']});
                })
                .catch((err) => {
                  console.log(err);
                })
              this.info.push({'sid': servers['s_list'][i], 'divs':[]});
              invoke('getChannels', {token: token, server: servers['s_list'][i], username: this.username})
              .then((res) => {
                let channels = JSON.parse(res)['c_list'];
                for(let j = 0; j<channels.length; j++) {
                  invoke('getMessages', {token: token, server:servers['s_list'][i], channel:channels[j]['channel_name'], username: this.username})
                    .then((res) => {
                      
                      this.info[this.info.findIndex(obj => obj.sid == servers['s_list'][i])].divs.push({'channelTag': channels[j]['channel_name'], 'messages': JSON.parse(res)['m_list'].sort(compareDate)});
                      this.divs.push({'sid':servers['s_list'][i], 'channelTag': channels[j]['channel_name'], 'messages': JSON.parse(res)['m_list'].sort(compareDate)})
                      })
                      .catch((err) => {
                        console.log('err');
                      });

                  }
                  this.done=true;
                  console.log(this.divs);
                })
                .catch((err) => {
                  console.log('err channels');
                }); 
                }
              console.log(this.info);
              console.log('INFO');
              console.log(this.info[this.info.findIndex(obj => obj.sid == this.sid)].divs);
          })
          .catch((err) => {
            console.log(err);
          });
          this.done = true;
      })
      .catch((err) => {
        console.log(err)
      });

    

        return {
          done:false,
          token:token,
          sid: "1",
          server:true,
          uservers:[{
            'sid':'1',
            'name':'Welcome',
            'img_url': 'https://onlinedi.vision/cdn/test/stest.jpeg',
            'desc':'Welcome!'
          }],
          info: [{
            'sid':'1',
            'divs':[{
              'channelTag':'welcome',
              'messages': [{
                'username':'System',
                'm_content':'Welcome to Division Online! Please feel at home! <3'
              }
              ]
              }
            ]
          }],
          username: 'ana',
          divs: [ {
              'sid':'1313',
              'channelTag':'info',
            }
          ],

      
          svusers: [{'img':'https://media1.tenor.com/m/viIU4ICp1N8AAAAd/dance.gif', 'username':'ana'}, {'img':'https://cdn.discordapp.com/attachments/1314144119010103319/1316541431518724096/IMG_4470.webp?ex=679f5181&is=679e0001&hm=b712b31c07433ade4cda2adb16863993be1c6421b16b24606b57e8267d3a239d&', 'username':'System'}, {'img': 'https://cdn.discordapp.com/attachments/556118918217859083/1335293396688048320/iu.png?ex=679fa462&is=679e52e2&hm=eb4e496a43cf6e52aad8833e027115767b84d9ec96b3eb9b755e7f3597e3f601&', 'username':'alex'}],
          name: '',
          textChannel: 'welcome',
          username: 'ana',
    };
  },
  methods: {
    addDiv(message) {
      console.log(message);
      if(this.sid=== '1') {message=''; this.name;}
      if(message==='')return;
      let sname = this.name;
      this.name = '';
      invoke('sendMessage', {token:this.token, channel: this.textChannel, server: this.sid,  m_content: message, username:'ana' }).then((res) => {
        
        this.info[this.info.findIndex(obj => obj.sid == this.sid)].divs[this.info[this.info.findIndex(obj => obj.sid == this.sid)].divs.findIndex(obj => obj.channelTag==this.textChannel)]['messages'].push({"username":this.username, "m_content":res});
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
     
      return month[Number(date.getMonth())] + ' ' + date.getDate() + ' '+ date.getFullYear() + ' ' + date.getHours() + ':' + date.getMinutes();
    },
    change_server(sv) {
      this.sid = sv;
      console.log(this.sid);
      this.textChannel = this.info[this.info.findIndex(obj => obj.sid == this.sid)].divs[0].channelTag;
    }
  },
  
};

</script>
<template >
  <main class="container" v-if="done">

    <form class="row" @submit.prevent="greet">
      <button id="send" style="font-size:15px;text-align:center;"><b>+</b></button>
      <input id="greet-input" v-model="name" placeholder="Type a message..." />
      <button id="send" type="submit" @click="addDiv(name)">Send</button>
    </form>
    <img src='https://media1.tenor.com/m/viIU4ICp1N8AAAAd/dance.gif' width='60px' height='60px' class='cui' style='margin-bottom:0px;'/>
  <img src='https://media1.tenor.com/m/viIU4ICp1N8AAAAd/dance.gif' width='60px' height='60px' class='cui' style='top:75px;margin-bottom:0px;'/>

    <div class = "column">  
      <template v-for="(sv, idx) in uservers">
        <img @click="change_server(sv['sid'])" v-bind:src="sv['img_url']" width='50px' height='50px' class='user-icon' style='margin-top: 0px;margin-top:10px;'/>
      </template>
    </div>
    

       <div class = "chanels">
         <div class ="server-header" style="border-bottom: 2px solid #1c0606; height: 60px;"> <h3> {{uservers[uservers.findIndex(obj => obj.sid == sid)].name}} </h3> </div>
           <template v-if="done">
             <template v-if="info[info.findIndex(obj => obj.sid == sid)].divs !== undefined">
              <template v-for="(div, index) in info[info.findIndex(obj => obj.sid == sid)].divs" :key="index">
                <button @click="change_channel(div['channelTag'])" class="channel_button"># {{div['channelTag']}}</button>
              </template>
            </template>
           </template>
        <button class="channel_button"> voice-channel</button>
        <button class="channel_button" style=" position: relative; text-align: center; padding-left: 0px; padding-right: 0px;"><b>+</b></button>

      </div>

      <div id="chat">

        <template v-if="done">
          <template v-for="(div, index) in info[info.findIndex(obj => obj.sid == sid)].divs[info[info.findIndex(obj => obj.sid == sid)].divs.findIndex(obj => obj.channelTag==textChannel)]['messages']" :key="index">
          <div class="comp-mess">
          <img v-bind:src="svusers[svusers.findIndex(obj => obj.username==div['username'])]['img']" width='40px' height='40px' class='user-icon' style='margin-top: 10px;margin-bottom:10px;'/>
          <div class="message">
            
            <div class="user" style="padding-top:6px;">
              <div ><RouterLink to='/profile'><i>{{div['username']}}</i></RouterLink></div>
              <div class="mdate" style="font-size:12px;padding-left:5px;padding-top:1px;"><i>{{get_date(Number(div['datetime']))}}</i></div>
            </div>
            <div v-if="div['m_content'].startsWith('https:')">
              <img v-bind:src="div['m_content']">
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
    .mdate {
      color: transparent;
  }
  .comp-mess:hover  .mdate  {
    color: #efa7aa;
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
  border-radius: 0px;
  border: 2px solid transparent;
  padding: 10px 10px;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0);
  width: 62px;
}

#send {
  cursor: pointer;
  background-color: #331515;
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
  background-color:#efa7aa; 
  color: #ffffff;
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
