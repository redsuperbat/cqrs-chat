"use client";

import axios from "axios";
import { useRouter } from "next/router";
import { FC, useEffect, useRef, useState } from "react";
import { useWebSocket } from "../../hooks/use-websocket";
import { UserStore } from "../../storage/user-store";
import { useSwr } from "../../swr/use-swr";
import type { Data } from "../api/chats/[id]";
import type { GetWebsocketUrlData } from "../api/websocket-url";

interface ChatMessage {
  message: string;
  sent_by: string;
  message_id: string;
}

const ChatMessage: FC<{ text: string; isMine: boolean }> = (props) => {
  const who = props.isMine ? "mine" : "theirs";
  return (
    <div className={`chat-message ${who}`}>
      <div className="text">{props.text}</div>
    </div>
  );
};

export default () => {
  const [message, setMessage] = useState<string>("");
  const [messages, setMessages] = useState<ChatMessage[]>([]);

  const id = useRouter().query.id;
  const inputRef = useRef<HTMLInputElement>(null);
  const { data: baseUrl } = useSwr<GetWebsocketUrlData>("/api/websocket-url");

  const { data: jsonMsg } = useWebSocket<ChatMessage>(
    baseUrl && id && `${baseUrl.url}/ws/?chat_id=${id}`
  );

  const { data: allPreviousMessages, isLoading } = useSwr<Data>(
    `/api/chats/${id}`
  );

  useEffect(() => {
    if (!jsonMsg) return;
    setMessages((it) => [jsonMsg, ...it]);
  }, [jsonMsg]);

  useEffect(() => {
    setMessages(allPreviousMessages?.messages ?? []);
  }, [allPreviousMessages]);

  const sendChatMessage = async () => {
    const username = UserStore.get()?.username;

    if (!message) {
      return;
    }

    if (!username) {
      return;
    }

    await axios.post("/api/send-chat-message", {
      message,
      chat_id: id,
      username,
    });
    setMessage("");
  };

  if (isLoading || !allPreviousMessages) {
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
              isMine={it.sent_by === UserStore.get()?.hashedUsername}
            />
          ))}
        </div>
        <div
          className="chat-lower-bound"
          onClick={() => inputRef.current?.focus()}
        >
          <div className="chat-input">
            <input
              type="text"
              ref={inputRef}
              value={message}
              onInput={(e) => setMessage(e.currentTarget.value)}
              onKeyUp={(e) => e.key === "Enter" && sendChatMessage()}
            />
            <button onClick={() => sendChatMessage()}>Send!</button>
          </div>
          <small className="text-center p-2">
            Remember, chatting is about making contact. If you want to read and
            not talk checkout my
            <br />
            <a
              href="https://github.com/redsuperbat"
              className="text-blue-500 font-bold underline"
            >
              github
            </a>
          </small>
        </div>
      </section>
    </div>
  );
};
