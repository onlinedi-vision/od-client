<script>
import { invoke } from '@tauri-apps/api/core';
import UserMessage from './UserMessage.vue';
import { shallowRef, ref } from "vue"
const inputRef = shallowRef()


export default {
  data() {
    let token="16ec6209e46700a7f29fea7b792b53b8f61d3705092bacf4eb853d9f497161b0";
   
    let server="1313";
    function compareDate(a, b) {
      if(Number(a.datetime) < Number(b.datetime)) {
        return 1;
      }
      if(Number(a.datetime) > Number(b.datetime)) {
        return -1;
      }
      return 0;

    }
    const socket = new WebSocket("wss://onlinedi.vision/chat/");
    socket.addEventListener('message', (event) => {
      const words = event.data.split(' ');
      const [msid, mchn, muser, ...mess] = words;
      console.log(msid);
      console.log(mchn);
      console.log(muser);
      console.log(mess);
this.info[this.info.findIndex(obj => obj.sid == msid)].divs[this.info[this.info.findIndex(obj => obj.sid == msid)].divs.findIndex(obj => obj.channelTag==mchn)]['messages'].unshift({"username":muser, "m_content":mess.join(' ')});
    });
    
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
            
            invoke('writeCredentials', {creds: JSON.stringify({'username': this.username, 'token': this.token }) });

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
        this.loggedin = false;
      });

    

        return {
          pli: true,
          loggedin: true,
          lusername: "",
          password: "",
          lemail: "",
          done:false,
          token:token,
          sid: "1",
          server:true,
          lError: false,
          lErrorText: "",
          createChannelPopUp: false,
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
              },
              {
                'username':'System',
                'm_content':'https://www.w3schools.com/html/mov_bbb.mp4'
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

      
          svusers: [{'img':'https://media1.tenor.com/m/viIU4ICp1N8AAAAd/dance.gif', 'username':'ana'}, {'img':'https://cdn.discordapp.com/attachments/1314144119010103319/1316541431518724096/IMG_4470.webp?ex=679f5181&is=679e0001&hm=b712b31c07433ade4cda2adb16863993be1c6421b16b24606b57e8267d3a239d&', 'username':'System'}, {'img': 'https://cdn.discordapp.com/attachments/556118918217859083/1335293396688048320/iu.png?ex=679fa462&is=679e52e2&hm=eb4e496a43cf6e52aad8833e027115767b84d9ec96b3eb9b755e7f3597e3f601&', 'username':'alesx'}],
          name: '',
          textChannel: 'welcome',
          username: 'ana',
          filename: '',
          nchn: '',
          createServerPopUp: false,
          newSvName: '',
          newShDesc: '',
          newImgUrl: '',
          joinsid: '',
          showSIDvar: false
    };
  },
  methods: {
    addDiv(message) {
      console.log(message);
      if(this.sid=== '1') {message=''; this.name;}
      if(message==='')return;
      let sname = this.name;
      this.name = '...';
      invoke('sendMessage', {token:this.token, channel: this.textChannel, server: this.sid,  m_content: message, username:'ana' }).then((res) => {
        console.log('taaa');
        this.name='';
      }).catch((err) =>{
      this.divs[this.divs.findIndex(obj => obj.channelTag===this.textChannel)]['messages'].unshift({"username":this.username, "m_content":err});
      });
      this.name='';
    },
    getUserServers() {
      invoke('getServers', {token: this.token, username: this.username})
          .then((res) => {
            console.log('LOL' + this.token);
            let servers = JSON.parse(res);
            this.token = servers['token'];
            
            invoke('writeCredentials', {creds: JSON.stringify({'username': this.username, 'token': this.token }) })
              .then((res) => console.log(res))
              .catch((err) => console.log(err));

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
              invoke('getChannels', {token: this.token, server: servers['s_list'][i], username: this.username})
              .then((res) => {
                let channels = JSON.parse(res)['c_list'];
                for(let j = 0; j<channels.length; j++) {
                  invoke('getMessages', {token: this.token, server:servers['s_list'][i], channel:channels[j]['channel_name'], username: this.username})
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

    }
    ,
    logIn() {
      invoke('logIn', {username: this.lusername, password: this.password})
        .then((res) => {
          console.log(res);
          

          let tokenJS = JSON.parse(res);
          this.token = tokenJS['token'];

          if(this.token === "") {
            this.lError=true;
            this.lErrorText="Authentification Failed";
            return;
          }

          this.username = this.lusername;
          this.password = "";
          this.loggedin = true;

          invoke('writeCredentials', {creds: JSON.stringify({'username': this.username, 'token': this.token }) });
          this.getUserServers();
        })
        .catch((err) => console.log(err));
    },
    signUp() {
      if(this.password.length < 8) {
            this.lError = true;
            this.lErrorText = "Password is too short";
            return
          }
      invoke('signUp', {username: this.lusername, password: this.password, email: this.lemail})
        .then((res) => {
          console.log(res);
          
          

          let tokenJS = JSON.parse(res);
          this.token = tokenJS['token'];

          if(this.token === "") {
            this.lError=true;
            this.lErrorText="User @" + this.lusername + " exists.";
            return;
          }

          this.username = this.lusername;
          this.password = "";
          this.loggedin = true;

          invoke('writeCredentials', {creds: JSON.stringify({'username': this.username, 'token': this.token }) });
          this.getUserServers();
        })
        .catch((err) => console.log(err));
    },
    changeLogIn() {
      this.pli = !this.pli;
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
    create_channel(svid) {
      this.createChannelPopUp=!this.createChannelPopUp;
      this.createServerPopUp=false;
    },
    createServer() {
      this.createServerPopUp=!this.createServerPopUp;
      this.createChannelPopUp=false;
    },
    change_server(sv) {
      this.sid = sv;
      console.log(this.sid);
      this.textChannel = this.info[this.info.findIndex(obj => obj.sid == this.sid)].divs[0].channelTag;
    },
    change_filename(ev) {
      this.filename = document.getElementById("file-upload").files[0];
      var reader = new FileReader();
      reader.readAsBinaryString(this.filename);
      reader.onload = function(ev) {
        console.log(ev.target.result);
        invoke('sendFile', {cont: ev.target.result})
          .then((res) => {
            let url = JSON.parse(res)['url'];
            this.addDiv(url);
          })
          .catch((res) =>  {
            console.log(res);
          });
      }
    },
    createChannel() {
      invoke('createChannel', {"username": this.username, "sid": this.sid, "token": this.token, "channel_name": this.nchn})
        .then((res) => {
          this.token = JSON.parse(res)['token'];
          this.info[this.info.findIndex(obj => obj.sid === this.sid)].divs.push({'channelTag': this.nchn, 'messages': []});

          invoke('writeCredentials', {creds: JSON.stringify({'username': this.username, 'token': this.token }) })
              .then((res) => console.log(res))
              .catch((err) => console.log(err));
          this.createChannelPopUp=!this.createChannelPopUp;
        })
        .catch((err) => {
          console.log(err);
        });
    },
    createSeverForReal() {
      invoke("createServer", {"img_url": this.newImgUrl, "desc": this.newShDesc, "name":this.newSvName, "username": this.username, "token": this.token})
        .then((res) => {
          this.token=JSON.parse(res)['token'];
          invoke('writeCredentials', {creds: JSON.stringify({'username': this.username, 'token': this.token }) })
            .then((res) => console.log(res))
            .catch((err) => console.log(err));
          this.createServerPopUp=!this.createServerPopUp;
        })
        .catch((err) => {
          console.log(err);
        });
    },
    showSID() {
      this.showSIDvar=!this.showSIDvar;
    },
    joinServer() {
      invoke('joinServer', {"username": this.username, "token": this.token, "sid": this.joinsid})
        .then((res) => {
          this.token=JSON.parse(res)['token'];
          invoke('writeCredentials', {creds: JSON.stringify({'username': this.username, 'token': this.token }) })
            .then((res) => console.log(res))
            .catch((err) => console.log(err));
          this.createServerPopUp=!this.createServerPopUp;

        })  
        .catch((err) => {
          console.log(err);
        })
    }
  },
  
};

</script>
<template >
  <main class="container" v-if="done && loggedin">

    <form class="row" @submit.prevent="greet">
      <input ref="inputRef" id="file-upload" type="file" @change="change_filename()"/>
      <label for="file-upload" class="custom-file-upload">
        <h3 style="margin-top: 10px;"class="fa fa-cloud-upload fa-plus"><b>+</b></h3>
      </label>
      <input  id="greet-input" v-model="name" placeholder="Type a message..." />
      <button id="send" type="submit" @click="addDiv(name)">Send</button>
    </form>
    <img src='https://media1.tenor.com/m/viIU4ICp1N8AAAAd/dance.gif' width='60px' height='60px' class='cui' style='margin-bottom:0px;'/>
    <button class="createSButton" @click="createServer()"><h2 style="margin-top: 12px">+</h2></button>
    <div  class = "column">  
      <template v-for="(sv, idx) in uservers">
        <img @click="change_server(sv['sid'])" v-bind:src="sv['img_url']" width='50px' height='50px' class='user-icon' style='margin-top: 0px;margin-top:10px;'/>
      </template>
    </div>
      
    <div v-if="showSIDvar" class="login" style="display:flex;flex-direction:row; left:100px; top:50px; height:30px; width:660px; z-index:999999;">
      <i style="font-size: 12px"> {{sid}} </i>
    </div>
       <div class = "chanels">
         <div class ="server-header" @click="showSID()" style="border-bottom: 2px solid #1c0606; height: 60px;"> <h3 style="margin-bottom: 0px;margin-top:5px"> {{uservers[uservers.findIndex(obj => obj.sid == sid)].name}} </h3> 

         </div>
           <template v-if="done">
             <template v-if="info[info.findIndex(obj => obj.sid == sid)].divs !== undefined">
              <template v-for="(div, index) in info[info.findIndex(obj => obj.sid == sid)].divs" :key="index">
                <template v-if="div['channelTag'] === textChannel">     
                  <button @click="change_channel(div['channelTag'])" class="channel_button" style="background-color: white"># {{div['channelTag']}}</button>
                </template>
                <template v-else>
                  <button @click="change_channel(div['channelTag'])" class="channel_button"># {{div['channelTag']}}</button>
                </template>
              </template>
            </template>
           </template>
        <button @click="create_channel(sid)" class="channel_button" style=" position: relative; text-align: center; padding-left: 0px; padding-right: 0px;"><b>+</b></button>

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
        <input class="csv" v-model="joinsid" placeholder="Server ID..."/>
        <button class="csvb" @click="joinServer()">Join Server</button>
      </div>



      <div id="chat">

        <template v-if="done">
          <template v-for="(div, index) in info[info.findIndex(obj => obj.sid == sid)].divs[info[info.findIndex(obj => obj.sid == sid)].divs.findIndex(obj => obj.channelTag==textChannel)]['messages']" :key="div">
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
      <main v-if="pli">
        
        <div v-if="lError" class="loginError"><h4>{{lErrorText}}</h4></div>
        <div class="login">
        <h1 style="margin-top: 100px;">Login</h1>
        <input class="username" v-model="lusername" placeholder="Username..." />      
        <input type="password" class="password" v-model="password" placeholder="Password..." /> 
        <button class="loginb" type="submit" @click="logIn()">Login</button>
        </div>
        <button class="change-login" type="submit" @click="changeLogIn()">I don't have an account</button>
      </main>
      <main v-else>

        <div v-if="lError" class="loginError"><h4>{{lErrorText}}</h4></div>
        <div class="login">
        <h1 style="margin-top: 100px;">Signup</h1>
        <input class="e-mail" v-model="lemail" placeholder="E-mail Address..." />      
        <input class="username" style="margin-top: 10px" v-model="lusername" placeholder="Username..." />      
        <input type="password" class="password" v-model="password" placeholder="Password..." /> 
        <button class="loginb" type="submit" @click="signUp()">SignUp</button>
        </div>
        <button class="change-login" type="submit" @click="changeLogIn()">I have an account</button>
      </main>
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
input[type="file"] {
    display: none;
}
.custom-file-upload {
    display: inline-block;
    width: 100px;
    cursor: pointer;
    background-color: #331515;
    height:43px;
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

  .csv {  
    background-color:#1c0606;
    color: #efa7aa;
  }
  .csv::placeholder {
    color: #efa7aa;
  }
  .csv:hover {
    border-color: #efa7aa;
  }

  .csvb {
height: 40px; text-align:center; background-color:#1c0606;
  }
  .csvb:hover {
    border-color: #efa7aa;
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
      top: 75px;
      left: 5px;
      height: calc(100vh - 155px);
      width: 60px;
      z-index: 9999;
      border-radius: 40px 40px 40px 40px;
      border: 2px solid transparent;
    }
.createSButton {
      transition: 0.2s; 
      background-color: #1c0606;
      position: absolute;
      top:calc(100vh - 70px);
      left: 5px;
      height: 60px;
      width: 60px;
      z-index: 9999;
      border-radius: 60px 60px 60px 60px;
      border: 2px solid transparent;
      text-align: center;
    }
    .login {
      
      background-color: #1c0606;
      position: absolute;
      display: flex;
      flex-direction: column;
      top: calc(50vh - 200px);
      left: calc(50vw - 150px);
      height: calc(100vh - 155px);
      width: 300px;
      height: 400px;
      z-index: 9999;
      text-align: center;
    }

    .loginError {
      
      background-color: #1c0606;
      position: absolute;
      display: flex;
      flex-direction: column;
      top: calc(50vh - 250px);
      left: calc(50vw - 150px);
      width: 300px;
      height: 50px;
      z-index: 9999;
      color: red;
      text-align: center;
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
    flex-direction: column-reverse;
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

.e-mail {
  background-color: #331515;/*#230d0e;*/ 
  color: #efa7aa;
  bottom: 0;
  margin-top: 40px;
  margin-left: 30px;
  width: 200px;
}

.username {
  background-color: #331515;/*#230d0e;*/ 
  color: #efa7aa;
  bottom: 0;
  margin-top: 90px;
  margin-left: 30px;
  width: 200px;
}

.password {
  background-color: #331515;/*#230d0e;*/ 
  color: #efa7aa;
  bottom: 0;
  margin-top: 10px;
  margin-left: 30px;
  width: 200px;
}

.loginb {
  cursor:pointer;
  background-color: #331515;/*#230d0e;*/ 
  color: #efa7aa;
  bottom: 0;
  margin-top: 10px;
  margin-left: 30px;
  height: 40px;
  width: 240px;
  justify-content: center;
  text-align: center;
}
.change-login {
  cursor:pointer;
  background-color: #331515;/*#230d0e;*/ 
  color: #efa7aa;
  bottom: 0;
  position: absolute;
  top: calc(50vh + 190px);
  left: calc(50vw - 180px);
  margin-top: 10px;
  margin-left: 30px;
  height: 40px;
  width: 300px;
  justify-content: center;
  text-align: center;
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
