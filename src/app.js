import { invoke } from '@tauri-apps/api/core';

const HEARTBEAT_INTERVAL = 25000;
const RECONNECT_INTERVAL = 5000;

export default {
  name: "App",
  data() {
    let token = null;

    invoke('get_local_token')
      .then((res) => {
        token = JSON.parse(res)['token'];
        this.username = JSON.parse(res)['username'];
        invoke('getservers', { token: token, username: this.username })
          .then((res) => {
            let servers = JSON.parse(res);
            this.token = servers.token;
            token = servers.token;
            this.initWebsocket(token);
            
            invoke('write_credentials', { creds: JSON.stringify({ 'username': this.username, 'token': this.token }) });

            for (let i = 0; i < servers['s_list'].length; i++) {
              invoke('get_server_info', { server_id: servers['s_list'][i] })
                .then((si) => {
                  let serverInfo = JSON.parse(si);
                  this.userServers.push({ 'serverID': servers['s_list'][i], 'name': serverInfo['name'], 'desc': serverInfo['desc'], 'img_url': serverInfo['img_url'] });
                })
                .catch((err) => {
                  console.log(err);
                })
              this.appState.push({ 'serverID': servers['s_list'][i], 'storedChannels': [], 'serverUsers': [] });
              invoke('getchannels', { host_url: 'https://onlinedi.vision/servers', token: token, server: servers['s_list'][i], username: this.username })
                .then((res) => {
                  let channels = JSON.parse(res)['c_list'];
                  for (let j = 0; j < channels.length; j++) {
                    invoke('getmessages', { host_url: 'https://onlinedi.vision/servers', token: token, server: servers['s_list'][i], channel: channels[j]['channel_name'], username: this.username })
                      .then((res) => {
                        this.appState[this.appState.findIndex(obj => obj.serverID == servers['s_list'][i])].storedChannels.push({ 'channelTag': channels[j]['channel_name'], 'messages': JSON.parse(res)['m_list'] });
                        this.storedChannels.push({ 'serverID': servers['s_list'][i], 'channelTag': channels[j]['channel_name'], 'messages': JSON.parse(res)['m_list'] })
                      })
                      .catch(() => {
                        this.appState[this.appState.findIndex(obj => obj.serverID == servers['s_list'][i])].storedChannels.push({ 'channelTag': channels[j]['channel_name'], 'messages': [] });
                        this.storedChannels.push({ 'serverID': servers['s_list'][i], 'channelTag': channels[j]['channel_name'], 'messages': [] })
                      });

                  }
                  invoke('get_server_users', { host_url: 'https://onlinedi.vision/servers', token: token, server: servers['s_list'][i], username: this.username })
                    .then((res) => {
                      let users = JSON.parse(res)['u_list'];
                      for (let user_iter = 0; user_iter < users.length; user_iter++) {
                        this.appState[this.appState.findIndex(obj => obj.serverID == servers['s_list'][i])].serverUsers.push(users[user_iter]);
                      }
                      this.done = true;
                    }).catch((err) => {
                      console.log(err);
                    });
                })
                .catch((err) => {
                  console.log(err);
                });
            }
			this.getOwnPfp();
          })
          .catch((err) => {
            console.log(err);
            this.loggedin = false;
          });
        this.done = true;
      })
      .catch((err) => {
        console.log(err)
        this.loggedin = false;
      });

    return {
      ws: null,
      heartbeat: null,
      logInSelected: true,
      loggedin: true,
      lusername: "",
      password: "",
      lemail: "",
      done: false,
      token: token,
      serverID: "1",
      server: true,
      lError: false,
      lErrorText: "",
      createChannelPopUp: false,
      userServers: [{
        'serverID': '1',
        'name': 'Welcome',
        'img_url': 'https://onlinedi.vision/cdn/test/stest.jpeg',
        'desc': 'Welcome!'
      }],
      appState: [{
        'serverID': '1',
        'storedChannels': [{
          'channelTag': 'welcome',
          'messages': [{
            'username': 'System',
            'm_content': 'Welcome to Division Online! Please feel at home! <3'
          },
          {
            'username': 'System',
            'm_content': 'https://www.w3schools.com/html/mov_bbb.mp4'
          }
          ]
        }
        ]
      }],
      username: 'noone',
      storedChannels: [{
        'serverID': '1313',
        'channelTag': 'info',
      }
      ],

      svusers: [
        { 'img': 'https://media1.tenor.com/m/viIU4ICp1N8AAAAd/dance.gif', 'username': 'ana' }
      ],
      message: '',
      textChannel: 'welcome',
      filename: '',
      nchn: '',
      createServerPopUp: false,
      newSvName: '',
      newShDesc: '',
      newImgUrl: '',
      joinserverID: '',
      showSIDvar: false,
      selectedFile: null,
      selectedFileUrl: '',
	  settingsOpen: false,
	  myPfp: 'https://media1.tenor.com/m/viIU4ICp1N8AAAAd/dance.gif'
    };
  },
  methods: {
    send_message(message) {
      if(message.length > 1200)return;
      
      const fileData = new FormData();
      const fileInput = document.getElementById("file-upload");
      const file = this.selectedFile || (fileInput?.files?.[0] ?? null);

      if (file) {

        fileData.append("file", file);

        fetch("https://onlinedi.vision:7377/upload", {
          method: 'POST',
          body: fileData
        }).then(response => {
          response.text().then(mess => {
            const to_append = "https://onlinedi.vision:7377" + mess.split(/\r?\n/).pop();
            message += to_append;

            if (this.serverID === '1') { message = ''; this.message; }
            if (message === '') return;
            this.message = '';

            if(this.ws !== null ) {
              this.ws.send(this.serverID + ':' + this.textChannel + ':' + this.username +':' + message);
            }
            invoke('sendmessage', { host_url: 'https://onlinedi.vision/servers', token: this.token, channel: this.textChannel, server: this.serverID, m_content: message, username: this.username })
            .then(() => {
              this.message = '';
              this.clearSelectedFile();
            }).catch((err) => {
              this.storedChannels[this.storedChannels.findIndex(obj => obj.channelTag === this.textChannel)]['messages'].unshift({ "username": this.username, "m_content": err });
            });
            this.message = this.clearSelectedFile();

          });
        })
          .catch(error => {
            console.error(error);
          });
        return;
      }

      if (this.serverID === '1') { message = ''; this.message; }
      if (message === '') return;
      this.message = '';

      if(this.ws !== null ) {
        this.ws.send(this.serverID + ':' + this.textChannel + ':' + this.username +':' + message);
      }
      invoke('sendmessage', { host_url: 'https://onlinedi.vision/servers', token: this.token, channel: this.textChannel, server: this.serverID, m_content: message, username: this.username })
      .then(() => {
        this.message = '';
      }).catch((err) => {
        this.storedChannels[this.storedChannels.findIndex(obj => obj.channelTag === this.textChannel)]['messages'].unshift({ "username": this.username, "m_content": err });
      });
      this.message = '';
    },
    getUserServers() {
      invoke('getservers', { token: this.token, username: this.username })
        .then((res) => {
          let servers = JSON.parse(res);
          this.token = servers['token'];


          invoke('write_credentials', { creds: JSON.stringify({ 'username': this.username, 'token': this.token }) })
            .then((res) => console.log(res))
            .catch((err) => console.log(err));

          for (let i = 0; i < servers['s_list'].length; i++) {
            invoke('get_server_info', { server_id: servers['s_list'][i] })
              .then((si) => {
                let sinfo = JSON.parse(si);
                this.userServers.push({ 'serverID': servers['s_list'][i], 'name': sinfo['name'], 'desc': sinfo['desc'], 'img_url': sinfo['img_url'] });
              })
              .catch((err) => {
                console.log(err);
              })
            this.appState.push({ 'serverID': servers['s_list'][i], 'storedChannels': [] });
            invoke('getchannels', { host_url: 'https://onlinedi.vision/servers', token: this.token, server: servers['s_list'][i], username: this.username })
              .then((res) => {
                let channels = JSON.parse(res)['c_list'];
                for (let j = 0; j < channels.length; j++) {
                  invoke('getmessages', { host_url: 'https://onlinedi.vision/servers', token: this.token, server: servers['s_list'][i], channel: channels[j]['channel_name'], username: this.username })
                    .then((res) => {

                      this.appState[this.appState.findIndex(obj => obj.serverID == servers['s_list'][i])].storedChannels.push({ 'channelTag': channels[j]['channel_name'], 'messages': JSON.parse(res)['m_list'] });
                      this.storedChannels.push({ 'serverID': servers['s_list'][i], 'channelTag': channels[j]['channel_name'], 'messages': JSON.parse(res)['m_list'] })
                    })
                    .catch((err) => {
                      console.log(err);
                    });

                }
                this.done = true;
              })
              .catch((err) => {
                console.log(err);
              });
          }
        })
        .catch((err) => {
          console.log(err);
        });
      this.done = true;

    }
    ,
    logIn(user, password) {
      this.lusername = user;
      this.password = password;
      invoke('login', { username: this.lusername, password: this.password })
        .then((res) => {

          let tokenJS = JSON.parse(res);
          this.token = tokenJS['token'];

          if (this.token === "") {
            this.lError = true;
            this.lErrorText = "Authentification Failed";
            return;
          }

          this.username = this.lusername;
          this.password = "";
          this.loggedin = true;

          invoke('write_credentials', { creds: JSON.stringify({ 'username': this.username, 'token': this.token }) });
          this.getUserServers();
        })
        .catch((err) => console.log(err));
    },
    signUp(user, password, email) {
      this.lusername = user;
      this.password = password;
      this.lemail = email;
      if (this.password.length < 8) {
        this.lError = true;
        this.lErrorText = "Password is too short";
        return
      }
      invoke('signup', { username: this.lusername, password: this.password, email: this.lemail })
        .then((res) => {
          let tokenJS = JSON.parse(res);
          this.token = tokenJS['token'];

          if (this.token === "") {
            this.lError = true;
            this.lErrorText = "User @" + this.lusername + " exists.";
            return;
          }

          this.username = this.lusername;
          this.password = "";
          this.loggedin = true;

          invoke('write_credentials', { creds: JSON.stringify({ 'username': this.username, 'token': this.token }) });
          this.getUserServers();
        })
        .catch((err) => console.log(err));
    },
    changeLogIn() {
      this.logInSelected = !this.logInSelected;
    },
	openSettings() {
	  this.settingsOpen = true;
	},
	closeSettings() {
	  this.settingsOpen = false;
	},
    get_messages(channel, server, token) {
      invoke('getmessages', { host_url: 'https://onlinedi.vision/servers', token: token, server: server, channel: channel })
        .then((res) => {
          console.log(res);
        })
        .catch((err) => {
          console.log(err);
        });
    },
    change_channel(newChan) {
      this.textChannel = newChan;
    },
    get_date(timestamp) {
      let date = new Date(Number(timestamp));
      let month = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

      return month[Number(date.getMonth())] + ' ' + date.getDate() + ' ' + date.getFullYear() + ' ' + date.getHours() + ':' + date.getMinutes();
    },
    create_channel() {
      this.createChannelPopUp = !this.createChannelPopUp;
      this.createServerPopUp = false;
    },
    create_channel_cancel() {
      this.createChannelPopUp = false;
      this.nchn = '';
    },
    createServer() {
      this.createServerPopUp = !this.createServerPopUp;
      this.createChannelPopUp = false;
    },
    createServerCancel() {
      this.createServerPopUp = false;
      this.newSvName = '';
      this.newImgUrl = '';
      this.newShDesc = '';
      this.joinserverID = '';
    },
    change_server(sv) {
      this.serverID = sv;
      this.textChannel = this.appState[this.appState.findIndex(obj => obj.serverID == this.serverID)].storedChannels[0].channelTag;
    },
    createChannel() {
      if(this.nchn.length > 15) return;
      invoke('create_channel', { host_url: 'https://onlinedi.vision/servers', "username": this.username, "server_id": this.serverID, "token": this.token, "channel_name": this.nchn })
        .then((res) => {
          this.token = JSON.parse(res)['token'];
          this.appState[this.appState.findIndex(obj => obj.serverID === this.serverID)].storedChannels.push({ 'channelTag': this.nchn, 'messages': [] });

          invoke('write_credentials', { creds: JSON.stringify({ 'username': this.username, 'token': this.token }) })
            .then((res) => console.log(res))
            .catch((err) => console.log(err));
          this.createChannelPopUp = !this.createChannelPopUp;
        })
        .catch((err) => {
          console.log(err);
        });
    },
	deleteChannel(sid, channelName) {
		invoke("delete_channel", { host_url: 'https://onlinedi.vision/servers', "username": this.username, "server_id": sid, "token": this.token, "channel_name": channelName })
		  .then((res) => {
			  if(res == "Channel deleted successfully") {
				this.appState[this.appState.findIndex(obj => obj.serverID === sid)].storedChannels =
				this.appState[this.appState.findIndex(obj => obj.serverID === sid)].storedChannels.filter(function(ch) {return ch.channelTag !== channelName;});
			  }
		  })
		.catch((err) => {
          console.log(err);
        });
	},
    createSeverForReal() {
      invoke("create_server", { "img_url": this.newImgUrl, "desc": this.newShDesc, "name": this.newSvName, "username": this.username, "token": this.token })
        .then((res) => {
          this.token  = JSON.parse(res)['token'];
          let new_sid = JSON.parse(res)['sid'];

          if(this.ws !== null ) {
            this.ws.send("UPDATE:"+this.token+":"+new_sid);
          }
          invoke('get_server_info', { server_id: new_sid })
            .then((si) => {
              let sinfo = JSON.parse(si);
              this.userServers.push({ 'serverID': new_sid, 'name': sinfo['name'], 'desc': sinfo['desc'], 'img_url': sinfo['img_url'] });
            })
            .catch((err) => {
              console.log(err);
            })

          this.appState.push({ 'serverID': new_sid, 'storedChannels': [], 'serverUsers': [] });
          invoke('getchannels', { host_url: 'https://onlinedi.vision/servers', token: this.token, server: new_sid, username: this.username })
            .then((res) => {
              let channels = JSON.parse(res)['c_list'];
              for (let j = 0; j < channels.length; j++) {
                this.appState[this.appState.findIndex(obj => obj.serverID == new_sid)].storedChannels.push({ 'channelTag': channels[j]['channel_name'], 'messages': []});
                this.storedChannels.push({ 'serverID': new_sid, 'channelTag': channels[j]['channel_name'], 'messages': []})
              }
              
              invoke('get_server_users', { host_url: 'https://onlinedi.vision/servers', token: this.token, server: new_sid, username: this.username })
                .then((res) => {
                  let users = JSON.parse(res)['u_list'];
                  for (let user_iter = 0; user_iter < users.length; user_iter++) {
                    this.appState[this.appState.findIndex(obj => obj.serverID == new_sid)].serverUsers.push(users[user_iter]);
                  }
                }).catch((err) => {
                  console.log(err);
                });              
            })
            .catch((err) => console.log(err));
                      
          invoke('write_credentials', { creds: JSON.stringify({ 'username': this.username, 'token': this.token }) })
            .then((res) => console.log(res))
            .catch((err) => console.log(err));
          this.createServerPopUp = !this.createServerPopUp;
        })
        .catch((err) => {
          console.log(err);
        });
    },
    showSID() {
      this.showSIDvar = !this.showSIDvar;
    },
    joinServer() {
      invoke('join_server', { host_url: 'https://onlinedi.vision/servers', "username": this.username, "token": this.token, "server_id": this.joinserverID })
        .then((res) => {
          this.token = JSON.parse(res)['token'];
          invoke('write_credentials', { creds: JSON.stringify({ 'username': this.username, 'token': this.token }) })
            .then((res) => console.log(res))
            .catch((err) => console.log(err));
          this.createServerPopUp = !this.createServerPopUp;

        })
        .catch((err) => {
          console.log(err);
        })
    },
	logOut() {
		this.loggedin=false;
		this.token='';
		this.username='';
		invoke('write_credentials', {creds: ''}).catch((err) => {
			console.log(err);
		});
		this.closeSettings();
	},
    onFileChange(e) {
      const files = e?.target?.files || [];
      if (this.selectedFileUrl) {
        try { URL.revokeObjectURL(this.selectedFileUrl); }
        catch (err) { console.log(err); }
      }
      if (files.length) {
        this.selectedFile = files[0];
        this.selectedFileUrl = URL.createObjectURL(this.selectedFile);
      } else {
        this.selectedFile = null;
        this.selectedFileUrl = '';
      }
    },
    clearSelectedFile() {
      if (this.selectedFileUrl) {
        try { URL.revokeObjectURL(this.selectedFileUrl); } catch (err) { console.log(err); }
      }
      this.selectedFile = null;
      this.selectedFileUrl = '';
      const input = document.getElementById('file-upload');
      if (input) input.value = '';
    },
	refreshToken(token) {
		this.token = token;
          invoke('write_credentials', { creds: JSON.stringify({ 'username': this.username, 'token': this.token }) })
            .then((res) => console.log(res))
            .catch((err) => console.log(err));
	},
	getOwnPfp() {
		invoke('get_profile_pic', { 'username': this.username, 'token': this.token})
			.then((res) => {
				this.refreshToken(JSON.parse(res)['token']);
				let url = JSON.parse(res)['img_url'];
				this.myPfp = url ? url : this.myPfp;
		}).catch((err) => {
          console.log(err);
        })
	},
	setOwnPfp(url) {
		invoke('set_profile_pic', { 'username': this.username, 'token': this.token, 'img_url': url})
			.then((res) => {
				this.refreshToken(JSON.parse(res)['token']);
				let url = JSON.parse(res)['img_url'];
				if(!url) console.log("WARN: problem encountered while setting pfp");
				this.myPfp = url ? url : this.myPfp;
		}).catch((err) => {
          console.log(err);
        })
	},
	initWebsocket(token){
    let message_ws = null;
    let ms_counter = 0;
    if(message_ws === null) {
      let message_ws = new WebSocket("wss://onlinedi.vision/wss?username="+this.username);
      message_ws.addEventListener("message", (event) => {
        if(ms_counter === 0 ) {
          ms_counter += 1;
          console.log("[WEBSOCKET] Message from server (" + ms_counter + ")", event.data);

          invoke('spellcheck', { token: token, username: this.username, key: event.data })
          .then((res) => {
            message_ws.send(res);
          }).catch((err) => {
            console.log(err);
          });
        } else if(ms_counter === 1) {
          ms_counter += 1;
          if(event.data === 'CONNECTED') {
            console.log('[WEBSOCKET CONNECTED]');
            message_ws.send('TOKEN:'+this.token);
          }
        } else if (ms_counter == 2) {
          ms_counter += 1;
          this.token = event.data;
        }else if(ms_counter > 2) {
          console.log("[WEBSOCKET MESSAGE]: " + event.data);

          if(event.data == "PONG") return;
          
          let splitm = event.data.split(':');
          let [sid, channel, username, ...message] = splitm;
          message = message.join(':');

          this.appState[
            this.appState.findIndex(obj => obj.serverID == sid)
          ]['storedChannels'][
              this.appState[
                this.appState.findIndex(obj => obj.serverID == sid)
              ]['storedChannels'].findIndex(obj => obj.channelTag === channel)
            ]['messages'].unshift({
                'username': username,
                'm_content': message,
                'datetime': Date.now()
              });
        }
      });
      message_ws.addEventListener("open", () => {
        if(this.reconnect != null) {
          clearInterval(this.reconnect);
        }
        this.heartbeat = setInterval(() => {
          if(message_ws.readyState === WebSocket.OPEN) {
            message_ws.send('PING');
          }
        }, HEARTBEAT_INTERVAL);
      });
      
      message_ws.addEventListener("close", () => {
        console.log('[WEBSOCKET CONNECTION LOST. ATTEMPTING TO RECONNECT]');
        clearInterval(this.heartbeat);
        setTimeout(() => { this.initWebsocket(this.token); }, RECONNECT_INTERVAL);
      });
      this.ws = message_ws;
    }
	}
  },
};
