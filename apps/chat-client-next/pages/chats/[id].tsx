import axios from "axios";
import { useRouter } from "next/router";
import { FC, useEffect, useState } from "react";
import useWebSocket from "react-use-websocket";
import useSWR from "swr";
import { UserStore } from "../../storage/user-store";
import type { Data } from "../api/chats/[id]";

const fetcher = (url: string) => axios.get(url).then((res) => res.data);

export default () => {
  const [message, setMessage] = useState<string>("");
  const [messages, setMessages] = useState<
    { message: string; sent_by: string; message_id: string }[]
  >([]);
  const router = useRouter();
  const { lastJsonMessage } = useWebSocket(`ws://localhost:8082/ws/`);

  const { data, isLoading, mutate } = useSWR<Data>(
    `/api/chats/${router.query.id}`,
    fetcher
  );

  useEffect(() => {
    setMessages((it) => [...it, lastJsonMessage]);
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
    <section>
      <div className="chat-messages">
        {messages.map((it) => (
          <ChatMessage
            key={it.message_id}
            text={it.message}
            isMine={it.sent_by === UserStore.get().hashedUsername}
          />
        ))}
      </div>
      <div className="chat-input">
        <input
          type="text"
          className="input"
          value={message}
          onInput={(e) => setMessage(e.currentTarget.value)}
          onKeyUp={(e) => e.key === "Enter" && sendChatMessage()}
        />
        <button className="button" onClick={() => sendChatMessage()}>
          Send!
        </button>
      </div>
    </section>
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
