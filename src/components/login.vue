<template>
<main v-if="logInSelected">
        
        <div v-if="lError" class="loginError"><h4>{{lErrorText}}</h4></div>
        <div class="login">
        <h1 style="margin-top: 100px;">Login</h1>
        <input class="username" v-model="lusername" placeholder="Username..." />      
        <input type="password" class="password" v-model="password" placeholder="Password..." /> 
        <button class="loginb" type="submit" @click="$emit('login', this.lusername, this.password)">Login</button>
        </div>
        <button class="change-login" type="submit" @click="$emit('changeLogIn')">I don't have an account</button>
      </main>
      <main v-else>

        <div v-if="lError" class="loginError"><h4>{{lErrorText}}</h4></div>
        <div class="login">
        <h1 style="margin-top: 100px;">Signup</h1>
        <input class="e-mail" v-model="lemail" placeholder="E-mail Address..." />      
        <input class="username" style="margin-top: 10px" v-model="lusername" placeholder="Username..." />      
        <input type="password" class="password" v-model="password" placeholder="Password..." /> 
        <button class="loginb" type="submit" @click="$emit('signup', this.lusername, this.password, this.lemail)">SignUp</button>
        </div>
        <button class="change-login" type="submit" @click="$emit('changeLogIn')">I have an account</button>
      </main>
</template>
<script>

export default {
	name: 'LogInWindow',
	props: {
		logInSelected: {
			type: Boolean,
			required: true
		}
	}
}
</script>