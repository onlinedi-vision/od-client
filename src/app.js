import { invoke } from '@tauri-apps/api/core';
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from '@tauri-apps/plugin-notification';

import { shallowRef, ref } from "vue"
import LogInWindow from './components/login.vue'
import SettingsWindow from './components/settings.vue'
export default {
  name: "App",
  components: {
    LogInWindow,
	SettingsWindow
  },
  data() {
    let token = "16ec6209e46700a7f29fea7b792b53b8f61d3705092bacf4eb853d9f497161b0";

    sendNotification({
      title: 'Welcome to Division Online!',
      body: "We're glad to have you!",
      attachments: [
        {
          id: 'image-1',
          url: 'asset:///app-icon.png',
        },
      ]
    });

    invoke('getLocalToken')
      .then((res) => {
        token = JSON.parse(res)['token'];
        this.username = JSON.parse(res)['username'];
        invoke('getServers', { token: token, username: this.username })
          .then((res) => {
            let servers = JSON.parse(res);
            this.token = servers.token;
            token = servers.token;

            invoke('writeCredentials', { creds: JSON.stringify({ 'username': this.username, 'token': this.token }) });

            for (let i = 0; i < servers['s_list'].length; i++) {
              invoke('getServerInfo', { server_id: servers['s_list'][i] })
                .then((si) => {
                  let serverInfo = JSON.parse(si);
                  this.userServers.push({ 'serverID': servers['s_list'][i], 'name': serverInfo['name'], 'desc': serverInfo['desc'], 'img_url': serverInfo['img_url'] });
                })
                .catch((err) => {
                  console.log(err);
                })
              this.appState.push({ 'serverID': servers['s_list'][i], 'storedChannels': [], 'serverUsers': [] });
              invoke('getChannels', { host_url: 'https://onlinedi.vision/servers', token: token, server: servers['s_list'][i], username: this.username })
                .then((res) => {
                  let channels = JSON.parse(res)['c_list'];
                  for (let j = 0; j < channels.length; j++) {
                    invoke('getMessages', { host_url: 'https://onlinedi.vision/servers', token: token, server: servers['s_list'][i], channel: channels[j]['channel_name'], username: this.username })
                      .then((res) => {
                        this.appState[this.appState.findIndex(obj => obj.serverID == servers['s_list'][i])].storedChannels.push({ 'channelTag': channels[j]['channel_name'], 'messages': JSON.parse(res)['m_list'] });
                        this.storedChannels.push({ 'serverID': servers['s_list'][i], 'channelTag': channels[j]['channel_name'], 'messages': JSON.parse(res)['m_list'] })
                      })
                      .catch((err) => {
                        this.appState[this.appState.findIndex(obj => obj.serverID == servers['s_list'][i])].storedChannels.push({ 'channelTag': channels[j]['channel_name'], 'messages': [] });
                        this.storedChannels.push({ 'serverID': servers['s_list'][i], 'channelTag': channels[j]['channel_name'], 'messages': [] })
                      });

                  }
                  invoke('getServerUsers', { host_url: 'https://onlinedi.vision/servers', token: token, server: servers['s_list'][i], username: this.username })
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
                  console.log('err channels');
                });
            }
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
      name: '',
      textChannel: 'welcome',
      username: 'ana',
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
	  settingsOpen: false
    };
  },
  methods: {
    send_message(message) {
      const fileElement = document.querySelector("#file-form");
      const fileData = new FormData();
      const fileInput = document.getElementById("file-upload");
      const file = this.selectedFile || (fileInput?.files?.[0] ?? null);

      if (file) {

        fileData.append("file", file);

        for (var pair of fileData.entries()) {
          console.log(pair[0] + '!!!!' + pair[1]);
        }
        fetch("https://onlinedi.vision:7377/upload", {
          method: 'POST',
          body: fileData
        }).then(response => {
          response.text().then(mess => {
            const to_append = "https://onlinedi.vision:7377" + mess.split(/\r?\n/).pop();
            message += to_append;
            console.log(message);

            if (this.serverID === '1') { message = ''; this.name; }
            if (message === '') return;
            let sname = this.name;
            this.name = '...';
            invoke('sendMessage', { host_url: 'https://onlinedi.vision/servers', token: this.token, channel: this.textChannel, server: this.serverID, m_content: message, username: this.username }).then((res) => {
              console.log('taaa');
              this.name = '';
              this.clearSelectedFile();
            }).catch((err) => {
              this.storedChannels[this.storedChannels.findIndex(obj => obj.channelTag === this.textChannel)]['messages'].unshift({ "username": this.username, "m_content": err });
            });
            this.name = this.clearSelectedFile();

          });
        })
          .catch(error => {
            console.error(error);
          });
        return;
      }


      if (this.serverID === '1') { message = ''; this.name; }
      if (message === '') return;
      let sname = this.name;
      this.name = '...';
      invoke('sendMessage', { host_url: 'https://onlinedi.vision/servers', token: this.token, channel: this.textChannel, server: this.serverID, m_content: message, username: this.username }).then((res) => {
        console.log('taaa');
        this.name = '';
        sendNotification({
          title: this.username + " #" + this.textChannel,
          body: message,
          attachments: [
            {
              id: 'image-1',
              url: 'asset:///app-icon.png',
            },
          ]
        });
      }).catch((err) => {
        this.storedChannels[this.storedChannels.findIndex(obj => obj.channelTag === this.textChannel)]['messages'].unshift({ "username": this.username, "m_content": err });
      });
      this.name = '';
    },
    getUserServers() {
      invoke('getServers', { token: this.token, username: this.username })
        .then((res) => {
          let servers = JSON.parse(res);
          this.token = servers['token'];

          invoke('writeCredentials', { creds: JSON.stringify({ 'username': this.username, 'token': this.token }) })
            .then((res) => console.log(res))
            .catch((err) => console.log(err));

          for (let i = 0; i < servers['s_list'].length; i++) {
            invoke('getServerInfo', { server_id: servers['s_list'][i] })
              .then((si) => {
                let sinfo = JSON.parse(si);
                this.userServers.push({ 'serverID': servers['s_list'][i], 'name': sinfo['name'], 'desc': sinfo['desc'], 'img_url': sinfo['img_url'] });
              })
              .catch((err) => {
                console.log(err);
              })
            this.appState.push({ 'serverID': servers['s_list'][i], 'storedChannels': [] });
            invoke('getChannels', { host_url: 'https://onlinedi.vision/servers', token: this.token, server: servers['s_list'][i], username: this.username })
              .then((res) => {
                let channels = JSON.parse(res)['c_list'];
                for (let j = 0; j < channels.length; j++) {
                  invoke('getMessages', { host_url: 'https://onlinedi.vision/servers', token: this.token, server: servers['s_list'][i], channel: channels[j]['channel_name'], username: this.username })
                    .then((res) => {

                      this.appState[this.appState.findIndex(obj => obj.serverID == servers['s_list'][i])].storedChannels.push({ 'channelTag': channels[j]['channel_name'], 'messages': JSON.parse(res)['m_list'] });
                      this.storedChannels.push({ 'serverID': servers['s_list'][i], 'channelTag': channels[j]['channel_name'], 'messages': JSON.parse(res)['m_list'] })
                    })
                    .catch((err) => {
                      console.log('err');
                    });

                }
                this.done = true;
                console.log(this.storedChannels);
              })
              .catch((err) => {
                console.log('err channels');
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
      console.log(this.lusername);
      console.log(this.password);
      invoke('logIn', { username: this.lusername, password: this.password })
        .then((res) => {
          console.log(res);


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

          invoke('writeCredentials', { creds: JSON.stringify({ 'username': this.username, 'token': this.token }) });
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
      invoke('signUp', { username: this.lusername, password: this.password, email: this.lemail })
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

          invoke('writeCredentials', { creds: JSON.stringify({ 'username': this.username, 'token': this.token }) });
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
      invoke('getMessages', { host_url: 'https://onlinedi.vision/servers', token: token, server: server, channel: channel })
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

      return month[Number(date.getMonth())] + ' ' + date.getDate() + ' ' + date.getFullYear() + ' ' + date.getHours() + ':' + date.getMinutes();
    },
    create_channel(svid) {
      this.createChannelPopUp = !this.createChannelPopUp;
      this.createServerPopUp = false;
    },
    createServer() {
      this.createServerPopUp = !this.createServerPopUp;
      this.createChannelPopUp = false;
    },
    change_server(sv) {
      this.serverID = sv;
      this.textChannel = this.appState[this.appState.findIndex(obj => obj.serverID == this.serverID)].storedChannels[0].channelTag;
    },
    createChannel() {
      invoke('createChannel', { host_url: 'https://onlinedi.vision/servers', "username": this.username, "server_id": this.serverID, "token": this.token, "channel_name": this.nchn })
        .then((res) => {
          this.token = JSON.parse(res)['token'];
          this.appState[this.appState.findIndex(obj => obj.serverID === this.serverID)].storedChannels.push({ 'channelTag': this.nchn, 'messages': [] });

          invoke('writeCredentials', { creds: JSON.stringify({ 'username': this.username, 'token': this.token }) })
            .then((res) => console.log(res))
            .catch((err) => console.log(err));
          this.createChannelPopUp = !this.createChannelPopUp;
        })
        .catch((err) => {
          console.log(err);
        });
    },
    createSeverForReal() {
      invoke("createServer", { "img_url": this.newImgUrl, "desc": this.newShDesc, "name": this.newSvName, "username": this.username, "token": this.token })
        .then((res) => {
          this.token = JSON.parse(res)['token'];
          invoke('writeCredentials', { creds: JSON.stringify({ 'username': this.username, 'token': this.token }) })
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
      invoke('joinServer', { host_url: 'https://onlinedi.vision/servers', "username": this.username, "token": this.token, "server_id": this.joinserverID })
        .then((res) => {
          this.token = JSON.parse(res)['token'];
          invoke('writeCredentials', { creds: JSON.stringify({ 'username': this.username, 'token': this.token }) })
            .then((res) => console.log(res))
            .catch((err) => console.log(err));
          this.createServerPopUp = !this.createServerPopUp;

        })
        .catch((err) => {
          console.log(err);
        })
    },
	logOut() {
		console.log("placeholder")
		token='';
		username='';
		loggedin=false;
		//location.reload();
	},
    onFileChange(e) {
      const files = e?.target?.files || [];
      if (this.selectedFileUrl) {
        try { URL.revokeObjectURL(this.selectedFileUrl); } catch (_) { }
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
        try { URL.revokeObjectURL(this.selectedFileUrl); } catch (_) { }
      }
      this.selectedFile = null;
      this.selectedFileUrl = '';
      const input = document.getElementById('file-upload');
      if (input) input.value = '';
    },

  },

};
