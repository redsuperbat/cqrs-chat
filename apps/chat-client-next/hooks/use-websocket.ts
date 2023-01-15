import { useEffect, useState } from "react";

const ConnectionState = {
  CLOSED: "CLOSED",
  OPEN: "OPEN",
  ERROR: "ERROR",
} as const;
type ConnectionState = typeof ConnectionState[keyof typeof ConnectionState];

export const useWebSocket = <T = unknown>(url?: string) => {
  const [data, setData] = useState<T>();
  const [connectionState, setConnectionState] = useState<ConnectionState>();

  const startWebsocket = () => {
    if (!url) return;
    // Create WebSocket connection.
    const socket = new WebSocket(url);

    socket.onmessage = (e) => {
      try {
        setData(JSON.parse(e.data));
      } catch {
        // Do nothing
      }
    };
    socket.onclose = () => {
      setConnectionState(ConnectionState.CLOSED);
      setTimeout(() => startWebsocket(), 1000);
    };
    socket.onerror = () => {
      setConnectionState(ConnectionState.ERROR);
    };
    socket.onopen = () => {
      setConnectionState(ConnectionState.OPEN);
    };

    return () => {
      socket.close();
    };
  };

  useEffect(() => {
    return startWebsocket();
  }, [url]);

  return { data, connectionState };
};
