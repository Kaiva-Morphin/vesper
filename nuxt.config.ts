// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2024-11-01',
  devtools: { enabled: false },
  app: {
    //pageTransition: { name: 'settings', mode: 'out-in' }
  },
  modules: [
    '@nuxtjs/tailwindcss',
    '@nuxt/icon',
    'nuxt-security',
    '@nuxtjs/device',
    '@nuxtjs/google-fonts',
    '@nuxt/image'
  ],
  css: [
    'public/variables.css',
    'assets/style.css',
    'assets/jet_brains.css',
  ],
  plugins: [
    'plugins/globalVariables'
  ],
  googleFonts: {
    families: {
      Inter: [100, 300, 400, 500, 700, 900]
    }
  }
})