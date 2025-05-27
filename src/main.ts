import { createApp } from "vue";
import App from "./App.vue";
//import { createWebHistory, createRouter, RouteRecordRaw } from 'vue-router';
//import UserMessage from "./UserMessage.vue"
//import { RouterLink } from 'vue-router'
//const { BASE_URL } = import.meta.env
//import UserProfile from "./UserProfile.vue"
/*const routes: Array<RouteRecordRaw> = [
  { path: '/profile', name:"profile", component: UserProfile },
  { path: '/', name: 'app', component: App}
]

const router: any = createRouter({
  history: createWebHistory(),
  routes,
});*/

createApp(App).mount("#app");

/*)
import App from "./App.vue";
import VueRouter from "vue-router";

import UserProfile from "./UserProfile.vue";

const routes = [
  {
    path: "/profile",
    component: UserProfile,
  },
  {
    path: "/test",
    component: App,
  }
];

const router = new VueRouter({
  mode: 'hash',
  base: '/',
  routes,
});
Vue.config.productionTip = false
Vue.use(VueRouter)
new Vue({
  router,
  render: h => h(App),
}).$mount('#app')

  */
