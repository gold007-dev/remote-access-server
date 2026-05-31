// server/middleware/proxy.ts
import { defineEventHandler } from "h3";
import { createProxyMiddleware } from "http-proxy-middleware";

export default defineEventHandler((event) => {
  const url = getRequestURL(event);
  if (url.pathname !== "/api/terminal/ws") {
    return; // Let other routes handle this
  }
  return new Promise<void>((resolve, reject) => {
    console.log("running");
    // This is the core proxy middleware
    const proxy = createProxyMiddleware({
      target: "ws://localhost:1234", // Your Rocket backend
      changeOrigin: true,
      ws: true, // Enable WebSocket proxying
      //   logLevel: 'debug',
    });

    // Use the raw Node.js request and response objects
    proxy(event.node.req, event.node.res, (err: any) => {
      if (err) reject(err);
      else resolve();
    });
  });
});
