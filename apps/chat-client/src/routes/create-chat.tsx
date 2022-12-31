import { useNavigate } from "@solidjs/router";
import { createEffect, createSignal } from "solid-js";
import { createRouteAction } from "solid-start";
import { Button } from "~/components/Button";
import { Input } from "~/components/Input";
import { CommandService } from "~/services/command-service";
import { UserStore } from "~/storage/user-store";

export default function CreateChat() {
  const [username, setUsername] = createSignal<string | undefined>(
    UserStore.get().username
  );

  const [chat, createChat] = createRouteAction(CommandService.createChat);
  const navigate = useNavigate();

  createEffect(() => {
    const result = chat.result;
    if (!result) return;
    UserStore.set({ username: username(), hash: result.data.data.user_id });
    navigate(`/chats/${result.data.data.chat_id}`);
  });

  return (
    <div class="grid place-items-center h-screen bg-gray-100">
      <div class="flex flex-col bg-white p-6 rounded-lg space-y-3">
        <h1>Start a chat with me! 🤘</h1>
        <Input
          label="Username"
          value={username()}
          onInput={(it) => setUsername(it)}
        />
        <Button
          label="Let's go 🚀"
          onClick={() => {
            const user = username();
            if (user) createChat(user);
          }}
        />
      </div>
    </div>
  );
}
