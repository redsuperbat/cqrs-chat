import axios from "axios";
import { useRouter } from "next/router";
import { useEffect, useState } from "react";
import { Info } from "../../components/info";
import { Tooltip } from "../../components/Tooltip";
import { UserStore } from "../../storage/user-store";

export default () => {
  const [username, setUsername] = useState<string>("");
  const [subject, setSubject] = useState<string>("");
  const [disabled, setDisabled] = useState(false);
  const router = useRouter();

  useEffect(() => {
    const username = UserStore.get()?.username;
    if (!username) return;
    setUsername(username);
    setDisabled(true);
  }, []);

  const clearUser = () => {
    UserStore.clear();
    setUsername("");
    setDisabled(false);
  };

  const createChat = async () => {
    if (!username) {
      return;
    }

    const { data } = await axios.post("/api/create-chat", {
      username,
      subject,
    });
    UserStore.set({
      user_id: data.data.user_id,
      username,
    });
    await router.push(`/chats/${data.data.chat_id}`);
  };

  return (
    <div id="main">
      <div className="card">
        <h3 className="text-xl font-bold">Chat with me! 🌟</h3>
        <div className="w-72">
          <div className="flex justify-between mb-1">
            <label
              htmlFor="username"
              className="mb-2 text-sm font-bold text-gray-700"
            >
              Username
            </label>
            <Tooltip text="Username is only stored client side">
              <Info />
            </Tooltip>
          </div>
          <div className="relative">
            <input
              id="username"
              type="text"
              className="w-full px-3 py-2 mb-3 leading-tight text-gray-700 border rounded shadow appearance-none focus:outline-none focus:shadow-outline"
              placeholder="Barry Plotter"
              value={username}
              onInput={(e) => setUsername(e.currentTarget.value)}
              disabled={disabled}
            />
            {disabled ? (
              <div
                onClick={() => clearUser()}
                className="absolute right-1 top-0 h-9 w-9 grid place-items-center text-red-500 hover:bg-gray-400 hover:bg-opacity-25 cursor-pointer rounded-full transition-all"
              >
                <svg
                  style={{
                    width: "24px",
                    height: "24px",
                  }}
                  viewBox="0 0 24 24"
                >
                  <path
                    fill="currentColor"
                    d="M9,3V4H4V6H5V19A2,2 0 0,0 7,21H17A2,2 0 0,0 19,19V6H20V4H15V3H9M7,6H17V19H7V6M9,8V17H11V8H9M13,8V17H15V8H13Z"
                  />
                </svg>
              </div>
            ) : null}
          </div>
          <label
            htmlFor="subject"
            className="mb-2 text-sm font-bold text-gray-700"
          >
            Subject
          </label>
          <input
            id="subject"
            type="text"
            className="w-full px-3 py-2 mb-3 leading-tight text-gray-700 border rounded shadow appearance-none focus:outline-none focus:shadow-outline"
            placeholder="Latest changes to the rust lang..."
            value={subject}
            onInput={(e) => setSubject(e.currentTarget.value)}
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
