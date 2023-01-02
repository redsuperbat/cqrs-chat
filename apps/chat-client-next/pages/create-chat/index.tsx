import axios from "axios";
import { useRouter } from "next/router";
import { useState } from "react";
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
    <main>
      <div className="card">
        <h3>Chat with me! 🌟</h3>
        <div className="input-container">
          <label>Username</label>
          <input
            type="text"
            value={username}
            onInput={(e) => setUsername(e.currentTarget.value)}
          />
        </div>
        <div>
          <button className="button" onClick={() => createChat()}>
            Lets go 🎙️
          </button>
        </div>
      </div>
    </main>
  );
};
