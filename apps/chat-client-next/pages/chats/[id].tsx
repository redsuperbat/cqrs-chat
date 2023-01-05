import axios from "axios";
import { useRouter } from "next/router";
import { FC, useEffect, useState } from "react";
import useWebSocket from "react-use-websocket";
import useSWR from "swr";
import { UserStore } from "../../storage/user-store";
import type { Data } from "../api/chats/[id]";
import type { GetWebsocketUrlData } from "../api/websocket-url";

const fetcher = (url: string) => axios.get(url).then((res) => res.data);

const createWebsocketUrl = (baseUrl?: string, chat_id?: string | string[]) => {
  if (!baseUrl || !chat_id) return "ws://0.0.0.0:8082/ws/";
  return `${baseUrl}/ws/?chat_id=${chat_id}`;
};

export default () => {
  const [message, setMessage] = useState<string>("");
  const [messages, setMessages] = useState<
    { message: string; sent_by: string; message_id: string }[]
  >([]);
  const router = useRouter();

  const { data: websocketUrl } = useSWR<GetWebsocketUrlData>(
    "/api/websocket-url",
    fetcher
  );

  const { lastJsonMessage } = useWebSocket(
    createWebsocketUrl(websocketUrl?.url, router.query.id)
  );

  const { data, isLoading } = useSWR<Data>(
    `/api/chats/${router.query.id}`,
    fetcher
  );

  useEffect(() => {
    if (!lastJsonMessage) return;
    setMessages((it) => [lastJsonMessage, ...it]);
  }, [lastJsonMessage]);

  useEffect(() => {
    setMessages(data?.messages ?? []);
  }, [data]);

  const sendChatMessage = async () => {
    if (!message) {
      return;
    }

    await axios.post("/api/send-chat-message", {
      message,
      chat_id: router.query.id,
      username: UserStore.get().username,
    });
    setMessage("");
  };

  if (isLoading || !data) {
    return (
      <div className="center-children">
        <div className="loading"></div>
      </div>
    );
  }

  return (
    <div className="chat-page-bg">
      <section className="chat-page">
        <div className="chat-messages">
          {messages.map((it) => (
            <ChatMessage
              key={it.message_id}
              text={it.message}
              isMine={it.sent_by === UserStore.get().hashedUsername}
            />
          ))}
        </div>
        <div className="chat-lower-bound">
          <div className="chat-input">
            <input
              type="text"
              value={message}
              onInput={(e) => setMessage(e.currentTarget.value)}
              onKeyUp={(e) => e.key === "Enter" && sendChatMessage()}
            />
            <button onClick={() => sendChatMessage()}>Send!</button>
          </div>
          <small>
            Remember, chatting is about making contact. If you want to read and
            not talk checkout my{" "}
            <a href="https://github.com/redsuperbat">github.</a>
          </small>
        </div>
      </section>
    </div>
  );
};

const ChatMessage: FC<{ text: string; isMine: boolean }> = (props) => {
  const who = props.isMine ? "mine" : "theirs";
  return (
    <div className={`chat-message ${who}`}>
      <div className="text">{props.text}</div>
    </div>
  );
};
