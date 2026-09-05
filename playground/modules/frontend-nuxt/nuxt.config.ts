// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2024-08-01',
  devtools: { enabled: true },
  app: {
    head: {
      title: 'Playground Nuxt Frontend',
      meta: [
        { name: 'description', content: 'MoldX playground Nuxt frontend' }
      ]
    }
  }
});
