import { invoke } from '@tauri-apps/api/core';
import { wsConnection } from './websocket.js';
import { reloadApp } from './AppWrapper.vue';

export default {
  name: "App",
  data() {
    return {
      ws: null,
      logInSelected: true,
      loggedin: true,
      lusername: "",
      password: "",
      lemail: "",
      done: false,
      token: null,
      serverID: "1",
      server: true,
      lError: false,
      lErrorText: "",
      createChannelPopUp: false,
      userServers: [{
        serverID: '1',
        name: 'Welcome',
        img_url: '',
        desc: 'Welcome!'
      }],
      appState: [{
        serverID: '1',
        storedChannels: [{
          channelTag: 'welcome',
          messages: [
            { username: 'System', m_content: 'Welcome to Division Online! Please feel at home! <3' },
            { username: 'System', m_content: 'https://www.w3schools.com/html/mov_bbb.mp4' }
          ]
        }]
      }],
      username: 'noone',
      storedChannels: [{
        serverID: '1313',
        channelTag: 'info',
      }],
      svusers: [
        { img: 'https://media1.tenor.com/m/viIU4ICp1N8AAAAd/dance.gif', username: 'ana' }
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
  async created() {
    await this.initializeApp();
  },
  methods: {
    async initializeApp() {
      try {
        const res = await invoke('get_local_token');
        const { token, username } = JSON.parse(res);
        this.username = username;
        await this.fetchServers(token, username);
        this.getOwnPfp();
        this.done = true;
      } catch (err) {
        console.log(err);
        this.loggedin = false;
      }
    },
    async fetchServers(token, username) {
      try {
        const res = await invoke('getservers', { token, username });
        const servers = JSON.parse(res);
        this.token = servers.token;
        await invoke('write_credentials', { creds: JSON.stringify({ username: this.username, token: this.token }) });
        this.initWebsocket(this.token);

        for (const serverID of servers.s_list) {
          await this.addServerInfo(serverID);
          await this.fetchChannelsAndUsers(serverID, token, username);
        }
      } catch (err) {
        console.log(err);
        this.loggedin = false;
      }
    },
    async addServerInfo(serverID) {
      try {
        const si = await invoke('get_server_info', { server_id: serverID });
        const serverInfo = JSON.parse(si);
        this.userServers.push({
          serverID,
          name: serverInfo.name,
          desc: serverInfo.desc,
          img_url: serverInfo.img_url
        });
        this.appState.push({ serverID, storedChannels: [], serverUsers: [] });
      } catch (err) {
        console.log(err);
      }
    },
    async fetchChannelsAndUsers(serverID, token, username) {
      try {
        const res = await invoke('getchannels', {
          host_url: 'https://api.onlinedi.vision/servers',
          token,
          server: serverID,
          username
        });
        const channels = JSON.parse(res).c_list;
        for (const channel of channels) {
          await this.fetchMessages(serverID, channel.channel_name, token, username);
        }
        await this.fetchServerUsers(serverID, token, username);
      } catch (err) {
        console.log(err);
      }
    },
    async fetchMessages(serverID, channelName, token, username) {
      try {
        const res = await invoke('getmessages', {
          host_url: 'https://api.onlinedi.vision/servers',
          token,
          server: serverID,
          channel: channelName,
          username
        });
        const messages = JSON.parse(res).m_list;
        this.addChannelMessages(serverID, channelName, messages);
      } catch {
        this.addChannelMessages(serverID, channelName, []);
      }
    },
    addChannelMessages(serverID, channelName, messages) {
      const stateIdx = this.appState.findIndex(obj => obj.serverID == serverID);
      if (stateIdx !== -1) {
        this.appState[stateIdx].storedChannels.push({ channelTag: channelName, messages });
      }
      this.storedChannels.push({ serverID, channelTag: channelName, messages });
    },
    async fetchServerUsers(serverID, token, username) {
      try {
        const res = await invoke('get_server_users', {
          host_url: 'https://api.onlinedi.vision/servers',
          token,
          server: serverID,
          username
        });
        const users = JSON.parse(res).u_list;
        const stateIdx = this.appState.findIndex(obj => obj.serverID == serverID);
        if (stateIdx !== -1) {
          this.appState[stateIdx].serverUsers = users;
        }
      } catch (err) {
        console.log(err);
      }
    },
    async send_message(message) {
      if (message.length > 1200) return;
      const fileInput = document.getElementById("file-upload");
      const file = this.selectedFile || (fileInput?.files?.[0] ?? null);

      if (file) {
        await this.uploadFileAndSendMessage(file, message);
        return;
      }
      await this.sendTextMessage(message);
    },
    async uploadFileAndSendMessage(file, message) {
      const fileData = new FormData();
      fileData.append("file", file);
      try {
        const response = await fetch("https://onlinedi.vision/ash/upload", {
          method: 'POST',
          body: fileData
        });
        const mess = await response.text();
        const to_append = "https://onlinedi.vision" + mess.split(/\r?\n/).pop();
        message += to_append;
        await this.sendTextMessage(message);
        this.clearSelectedFile();
      } catch (error) {
        console.error(error);
      }
    },
    async sendTextMessage(message) {
      if (this.serverID === '1') message = '';
      if (message === '') return;
      this.message = '';
      if (this.ws !== null) {
        this.ws.send(`${this.serverID}:${this.textChannel}:${this.username}:${message}`);
      }
      try {
        await invoke('sendmessage', {
          host_url: 'https://api.onlinedi.vision/servers',
          token: this.token,
          channel: this.textChannel,
          server: this.serverID,
          m_content: message,
          username: this.username
        });
        this.message = '';
      } catch (err) {
        const idx = this.storedChannels.findIndex(obj => obj.channelTag === this.textChannel);
        if (idx !== -1) {
          this.storedChannels[idx].messages.unshift({ username: this.username, m_content: err });
        }
      }
    },
    async getUserServers() {
      await this.fetchServers(this.token, this.username);
      this.done = true;
    },
    async logIn(user, password) {
      this.lusername = user;
      this.password = password;
      try {
        const res = await invoke('login', { username: this.lusername, password: this.password });
        const tokenJS = JSON.parse(res);
        this.token = tokenJS.token;
        if (!this.token) {
          this.lError = true;
          this.lErrorText = "Authentification Failed";
          return;
        }
        this.username = this.lusername;
        this.password = "";
        this.loggedin = true;
        await invoke('write_credentials', { creds: JSON.stringify({ username: this.username, token: this.token }) });
        setTimeout(reloadApp, 500);
      } catch (err) {
        console.log(err);
      }
    },
    async signUp(user, password, email) {
      this.lusername = user;
      this.password = password;
      this.lemail = email;
      if (this.password.length < 8) {
        this.lError = true;
        this.lErrorText = "Password is too short";
        return;
      }
      try {
        const res = await invoke('signup', { username: this.lusername, password: this.password, email: this.lemail });
        const tokenJS = JSON.parse(res);
        this.token = tokenJS.token;
        if (!this.token) {
          this.lError = true;
          this.lErrorText = `User @${this.lusername} exists.`;
          return;
        }
        this.username = this.lusername;
        this.password = "";
        this.loggedin = true;
        await invoke('write_credentials', { creds: JSON.stringify({ username: this.username, token: this.token }) });
        await this.getUserServers();
      } catch (err) {
        console.log(err);
      }
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
    async get_messages(channel, server, token) {
      try {
        const res = await invoke('getmessages', {
          host_url: 'https://api.onlinedi.vision/servers',
          token,
          server,
          channel
        });
        console.log(res);
      } catch (err) {
        console.log(err);
      }
    },
    change_channel(newChan) {
      this.textChannel = newChan;
    },
    get_date(timestamp) {
      const date = new Date(Number(timestamp));
      const month = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
      return `${month[date.getMonth()]} ${date.getDate()} ${date.getFullYear()} ${date.getHours()}:${date.getMinutes().toString().padStart(2, '0')}`;
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
      const state = this.appState.find(obj => obj.serverID == this.serverID);
      if (state && state.storedChannels.length > 0) {
        this.textChannel = state.storedChannels[0].channelTag;
      }
    },
    async createChannel() {
      if (this.nchn.length > 15) return;
      try {
        const res = await invoke('create_channel', {
          host_url: 'https://api.onlinedi.vision/servers',
          username: this.username,
          server_id: this.serverID,
          token: this.token,
          channel_name: this.nchn
        });
        this.token = JSON.parse(res).token;
        const stateIdx = this.appState.findIndex(obj => obj.serverID === this.serverID);
        if (stateIdx !== -1) {
          this.appState[stateIdx].storedChannels.push({ channelTag: this.nchn, messages: [] });
        }
        await invoke('write_credentials', { creds: JSON.stringify({ username: this.username, token: this.token }) });
        this.createChannelPopUp = !this.createChannelPopUp;
      } catch (err) {
        console.log(err);
      }
    },
    async deleteChannel(sid, channelName) {
      try {
        const res = await invoke("delete_channel", {
          host_url: 'https://api.onlinedi.vision/servers',
          username: this.username,
          server_id: sid,
          token: this.token,
          channel_name: channelName
        });
        if (res == "Channel deleted successfully") {
          const stateIdx = this.appState.findIndex(obj => obj.serverID === sid);
          if (stateIdx !== -1) {
            this.appState[stateIdx].storedChannels = this.appState[stateIdx].storedChannels.filter(ch => ch.channelTag !== channelName);
          }
        }
      } catch (err) {
        console.log(err);
      }
    },
    async createSeverForReal() {
      try {
        const res = await invoke("create_server", {
          img_url: this.newImgUrl,
          desc: this.newShDesc,
          name: this.newSvName,
          username: this.username,
          token: this.token
        });
        const { token, sid } = JSON.parse(res);
        this.token = token;
        if (this.ws !== null) {
          this.ws.send(`UPDATE:${this.token}:${sid}`);
        }
        await this.addServerInfo(sid);
        await this.fetchChannelsAndUsers(sid, this.token, this.username);
        await invoke('write_credentials', { creds: JSON.stringify({ username: this.username, token: this.token }) });
        this.createServerPopUp = !this.createServerPopUp;
      } catch (err) {
        console.log(err);
      }
    },
    showSID() {
      this.showSIDvar = !this.showSIDvar;
    },
    async joinServer() {
      try {
        const res = await invoke('join_server', {
          host_url: 'https://api.onlinedi.vision/servers',
          username: this.username,
          token: this.token,
          server_id: this.joinserverID
        });
        this.token = JSON.parse(res).token;
        await invoke('write_credentials', { creds: JSON.stringify({ username: this.username, token: this.token }) });
        this.createServerPopUp = !this.createServerPopUp;
      } catch (err) {
        console.log(err);
      }
    },
    logOut() {
      this.loggedin = false;
      this.token = '';
      this.username = '';
      invoke('write_credentials', { creds: '' }).catch(err => console.log(err));
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
    async refreshToken(token) {
      this.token = token;
      try {
        await invoke('write_credentials', { creds: JSON.stringify({ username: this.username, token: this.token }) });
      } catch (err) {
        console.log(err);
      }
    },
    async getOwnPfp() {
      try {
        const res = await invoke('get_profile_pic', { username: this.username, token: this.token });
        await this.refreshToken(JSON.parse(res).token);
        const url = JSON.parse(res).img_url;
        this.myPfp = url ? url : this.myPfp;
      } catch (err) {
        console.log(err);
      }
    },
    async setOwnPfp(url) {
      try {
        const res = await invoke('set_profile_pic', { username: this.username, token: this.token, img_url: url });
        await this.refreshToken(JSON.parse(res).token);
        const newUrl = JSON.parse(res).img_url;
        if (!newUrl) console.log("WARN: problem encountered while setting pfp");
        this.myPfp = newUrl ? newUrl : this.myPfp;
      } catch (err) {
        console.log(err);
      }
    },
    initWebsocket() {
      this.ws = new wsConnection(this.username);
      this.ws.addEventListener("reqHandshake", () => {
        this.ws.handshake(this.token).then(res => {
          this.token = res;
          this.ws.startReceive();
        });
      });
      this.ws.connectWebsocket();
      this.ws.addEventListener("message", this.receiveMessage);
    },
    receiveMessage(event) {
      const splitm = event.detail.split(':');
      const [sid, channel, username, ...messageArr] = splitm;
      const message = messageArr.join(':');
      const stateIdx = this.appState.findIndex(obj => obj.serverID == sid);
      if (stateIdx !== -1) {
        const channelIdx = this.appState[stateIdx].storedChannels.findIndex(obj => obj.channelTag === channel);
        if (channelIdx !== -1) {
          this.appState[stateIdx].storedChannels[channelIdx].messages.unshift({
            username,
            m_content: message,
            datetime: Date.now()
          });
        }
      }
    }
  }
};
