import { createApp } from 'vue'
import App from './App.vue'
import Routes from './routes'
import './assets/icon/iconfont.js'
import './assets/icon/iconfont.css'
import MenuBar from './components/MenuBar.vue'
const app = createApp(App)
app.use(Routes)
app.component('menu-bar', MenuBar);
app.mount('#app')
