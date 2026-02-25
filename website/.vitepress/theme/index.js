import DefaultTheme from 'vitepress/theme'
import ZedChat from './components/ZedChat.vue'
import './style.css'

export default {
    extends: DefaultTheme,
    enhanceApp({ app }) {
        app.component('ZedChat', ZedChat)
    }
}
