import tailwindcss from "@tailwindcss/vite";

// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: "2025-07-15",
  devtools: { enabled: true },
  vite: {
    plugins: [tailwindcss()],
    optimizeDeps: {
      include: [
        "@vue/devtools-core",
        "@vue/devtools-kit",
        "@vue/devtools-core",
        "@vue/devtools-kit",
        "@xterm/xterm",
        "@xterm/addon-fit",
        "@xterm/addon-attach",
      ],
    },
    // server: {
    //   proxy: {
    //     "/api": "http://172.17.0.1:1234",
    //     // Proxy WebSocket requests
    //     // "/terminal/ws": {
    //     //   target: "ws://localhost:1234",
    //     //   ws: true, // Important for WebSockets
    //     //   changeOrigin: true,
    //     //   rewrite: (path) => {
    //     //     console.log("running");
    //     //     return path;
    //     //   },
    //     // },
    //   },
    // },
  },
  css: ["~/assets/css/main.css"],
  modules: ["@pinia/nuxt"],
  ssr: false,
  nitro: {
    experimental: {
      websocket: true,
    },
    routeRules: {
      "/api/**": {
        proxy: "http://host.docker.internal:1234/api/**",
      },
    },
  },
});
