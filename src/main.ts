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
/*;
import App from "./App.vue";
import VueRouter from "vue-router";

import UserProfile from "./UserProfile.vue";

const routes = [
  {
    path: "/profile",
    component: UserProfile,
  }
];

const router = new VueRouter({
  mode: "hash",
  base: "/",
  routes,
});
createApp(App).mount("#app");
*/
