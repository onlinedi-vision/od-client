<script>

export default {
  data() {
    return {
      divs: [ {
      'channelTag':'text-channel1',
      'messages':
        [
          {'div':'doru911', 'mess':'ma duc sa ma culc'},
          {'div':'rub', 'mess':'https://media1.tenor.com/m/SG2Y2dkZvqoAAAAd/ori.gif'},
          {'div':'W', 'mess':'https://media1.tenor.com/m/cPuWFCXAUuMAAAAd/cat-hug.gif'},
          {'div':'doru911', 'mess':'https://media1.tenor.com/m/AmUL0BkdH5QAAAAd/hug-couple.gif'}
        ]
      },
      {
      'channelTag':'text-channel2',
      'messages': [
          {'div':'rub', 'mess':'test'},
        ]
      },
      {
      'channelTag':'text-channel3',
      'messages': [
        ]
      },
      {
      'channelTag':'text-channel4',
      'messages': [
        ]
      }
      ],

      
      svusers: [{'img':'https://media1.tenor.com/m/viIU4ICp1N8AAAAd/dance.gif', 'username':'W'}, {'img':'https://cdn.discordapp.com/attachments/1314144119010103319/1316541431518724096/IMG_4470.webp?ex=679f5181&is=679e0001&hm=b712b31c07433ade4cda2adb16863993be1c6421b16b24606b57e8267d3a239d&', 'username':'rub'}, {'img': 'https://cdn.discordapp.com/attachments/556118918217859083/1335293396688048320/iu.png?ex=679fa462&is=679e52e2&hm=eb4e496a43cf6e52aad8833e027115767b84d9ec96b3eb9b755e7f3597e3f601&', 'username':'doru911'}],
      name: '',
      textChannel: 'text-channel1',
      username: 'W',
    };
  },
  methods: {
    async addDiv(message) {
      this.divs[this.divs.findIndex(obj => obj.channelTag==this.textChannel)]['messages'].push({"div":this.username, "mess":message});
      console.log('here');
      this.name='';
      //await invoke('testRust');
    },
    change_channel(newChan) {
      this.textChannel = newChan;
    }
  }
};

</script>
<template>
  <main>

  </main> 
  <main class="container">

    <form class="row" @submit.prevent="greet">
      <input id="greet-input" v-model="name" placeholder="Type a message..." />
      <button id="send" type="submit" @click="addDiv(name)">Send</button>
    </form>

    <div class = "column">  
    </div>
    


    <div class = "chanels">
      <div style="border-bottom: 2px solid #1c0606; height: 60px;"> <h3> Server Name </h3> </div>
      <button @click="change_channel('text-channel1')"># text-channel1</button>
      <button @click="change_channel('text-channel2')"># text-channel2</button>
      <button @click="change_channel('text-channel3')"># text-channel3</button>
      <button @click="change_channel('text-channel4')"># text-channel4</button>
      <button>> voice-channel</button>
    </div>

    <div id="chat">
      <template v-for="(div, index) in divs[divs.findIndex(obj => obj.channelTag==textChannel)]['messages']" :key="index">
        <div class="comp-mess">
        <img v-bind:src="svusers[svusers.findIndex(obj => obj.username==div['div'])]['img']" width='40px' height='40px' class='user-icon' style='margin-top: 10px;margin-bottom:10px;'/>
        <div class="message">
          
          <div class="user">
            <div><RouterLink to='/'>{{div['div']}}</RouterLink></div>
            <div style="font-size:10px;padding-left:5px;">08-01-2025 9:15</div>
          </div>
          <div v-if="div['mess'].startsWith('https:')">
            <img v-bind:src="div['mess']">
          </div>
          <div v-else>
            {{div['mess']}}
          </div>
        </div>
        </div>
      </template>
    </div>
    <div id="channel-header">
    </div>
    <!--<p>{{ greetMsg }}</p>-->

    <div class = "server-users">
      <template v-for="(svuser,index) in svusers" :key="index">
          <img v-bind:src="svuser['img']" width='50px' class="user-icon"/>
      </template>
    </div>
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
  border: 2px solid transparent;
  margin-top: 10px;
}

.logo.tauri:hover {
  filter: drop-shadow(0 0 2em #24c8db);
}

.row {
  display: flex;
  justify-content: center;
  position: absolute;
  bottom: 0.4vw;
  right: 0.5vw;
  left: 20vw;
  width: 79vw;
}

.column {
  background-color: #1c0606;
  position: absolute;
  top: 5px;
  left: 0.5vw;
  height: 98.5vh;
  width: 4vw;
  border-radius: 40px 0px 0px 40px;
  border: 2px solid transparent;
}

.server-users {
  background-color: #1c0606;
  position: absolute;
  top: 5px;
  right: 0.5vw;
  height: 93vh;
  width: 4vw;
  border-radius: 0px 0px 40px 40px;
  border: 2px solid transparent;
}

.chanels {
  background-color: #331515;
  position: absolute;
  top: 5px;
  left: 4.8vw;
  height: 98.5vh;
  width: 14.9vw;
  border: 2px  solid transparent;
  display: flex;
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
  left: 20vw;
  width: 75vw;
  top: 68px;
  overflow-y: scroll;
  height: 88vh;
}

#channel-header {
  justify-content: center;
  text-align: left;
  position: absolute;
  top: 5px;
  border-bottom: 2px solid #1c0606;
  width: 75vw;
  height: 63px;
  left: 20vw;
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
  background-color:#422828;

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
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
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
  border-color: #efa7aa;
}

.server-users:hover {
  border-color: #efa7aa;
}

.user-icon:hover {
  border-color: #efa7aa;
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
