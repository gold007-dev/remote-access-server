// server/routes/api/terminal/ws.ts
// import { defineWebSocketHandler } from 'nitropack'
import WebSocket from "ws";

export default defineWebSocketHandler({
  open(peer) {
    // 1. Get the 'Cookie' header from the client's handshake request.
    const cookieHeader = peer.request.headers.get("cookie");
    if (!cookieHeader) {
      console.warn("Missing cookie header, closing connection.");
      peer.close(1008, "Unauthorized");
      return;
    }

    // 2. Establish the connection to your Rocket backend, forwarding the cookie.
    const backendSocket = new WebSocket("ws://localhost:1234/api/terminal/ws", {
      headers: {
        Cookie: cookieHeader,
      },
    });

    // 3. Store the backend connection in the peer's context.
    peer.context.backendSocket = backendSocket;

    // 4. Relay messages from the backend to the client.
    backendSocket.onmessage = (event) => {
      peer.send(event.data);
    };

    // 5. Handle connection closure.
    backendSocket.onclose = () => peer.close();
    backendSocket.onerror = (err) => {
      console.error("Backend WebSocket error:", err);
      peer.close();
    };
  },

  message(peer, message) {
    // Relay messages from the client to the backend.
    if (peer.context.backendSocket?.readyState === WebSocket.OPEN) {
      peer.context.backendSocket.send(message);
    }
  },

  close(peer) {
    // Clean up the backend connection.
    peer.context.backendSocket?.close();
  },
});
