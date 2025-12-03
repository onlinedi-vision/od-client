import { invoke } from '@tauri-apps/api/core';
const HEARTBEAT_INTERVAL = 25000;
const RECONNECT_INTERVAL = 5000;
const HANDSHAKE_LENGTH = 2;
export class wsConnection extends EventTarget{
  message_ws = null;
  heartbeat = null;
  username = null;
  
  constructor(username) {
    super();
    this.username = username;
    this.connectWebsocket();
  }
  connectWebsocket() {
    this.message_ws = new WebSocket("wss://onlinedi.vision/wss?username="+this.username);
    this.message_ws.addEventListener("open", () => {
      this.heartbeat = setInterval(() => {
        if(this.message_ws.readyState === WebSocket.OPEN) {
          this.message_ws.send('PING');
        }
      }, HEARTBEAT_INTERVAL);
    });
  }
  handshake(token, username = this.username) {
    return new Promise(resolve => {
      let ms_counter = 0;
      function handler(event) {
        if(ms_counter === 0 ) {
          ms_counter += 1;
          console.log("[WEBSOCKET] Message from server (" + ms_counter + ")", event.data);
          invoke('spellcheck', { token: token, username: username, key: event.data })
          .then((res) => {
            this.send(res);
          }).catch((err) => {
            console.log(err);
          });
        } else if(ms_counter === 1) {
          ms_counter += 1;
          if(event.data === 'CONNECTED') {
            console.log('[WEBSOCKET CONNECTED]');
            this.send('TOKEN:'+token);
          }
        } else if (ms_counter == 2) {
          ms_counter += 1;
          this.removeEventListener("message", handler);
          resolve(event.data);
        }
      }
      this.message_ws.addEventListener("message", handler);
    }
  );
  }
  send(msg) {
    this.message_ws.send(msg);
  }
}