// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2024-11-01',
  devtools: { enabled: true },
  app: {
    pageTransition: { name: 'page', mode: 'out-in' }
  },
  modules: [
    '@nuxtjs/tailwindcss',
    '@nuxt/icon',
    'nuxt-security',
    '@nuxtjs/device',
    '@nuxtjs/google-fonts'
  ],
  css: [
    'assets/variables.css',
    'assets/style.css',
    'assets/jet_brains.css',
  ],
  googleFonts: {
    families: {
      Inter: [100, 300, 400, 500, 700, 900]
    }
  }
})