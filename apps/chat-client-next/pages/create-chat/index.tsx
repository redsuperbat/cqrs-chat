import axios from "axios";
import { useRouter } from "next/router";
import { useState } from "react";
import { Info } from "../../components/info";
import { Tooltip } from "../../components/tooltip";
import { UserStore } from "../../storage/user-store";

export default () => {
  const [username, setUsername] = useState<string>("");
  const router = useRouter();

  const createChat = async () => {
    if (!username) {
      return;
    }

    const { data } = await axios.post("/api/create-chat", {
      username,
    });
    UserStore.set({
      hashedUsername: data.data.user_id,
      username,
    });
    await router.push(`/chats/${data.data.chat_id}`);
  };

  return (
    <div id="main">
      <div className="card">
        <h3>Chat with me! 🌟</h3>
        <div className="input-container">
          <div className="input-container-label-wrapper">
            <label>Username</label>
            <Tooltip text="Username is only stored client side">
              <Info />
            </Tooltip>
          </div>
          <input
            type="text"
            value={username}
            onInput={(e) => setUsername(e.currentTarget.value)}
          />
        </div>
        <div>
          <button className="smart-blue-button" onClick={() => createChat()}>
            🎙️ Lets talk 🎙️
          </button>
        </div>
      </div>
    </div>
  );
};
