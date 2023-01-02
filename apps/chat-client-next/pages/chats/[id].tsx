import axios from "axios";
import { useRouter } from "next/router";
import { FC, useState } from "react";
import useSWR from "swr";
import { UserStore } from "../../storage/user-store";
import { Data } from "../api/chats/[id]";

const fetcher = (url: string) => axios.get(url).then((res) => res.data);

export default () => {
  const [message, setMessage] = useState<string>("");
  const router = useRouter();

  const { data, isLoading, mutate } = useSWR<Data>(
    `/api/chats/${router.query.id}`,
    fetcher
  );

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
    await mutate();
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
        {data.messages.map((it) => (
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
