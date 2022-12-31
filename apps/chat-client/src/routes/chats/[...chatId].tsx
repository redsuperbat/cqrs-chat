import { createSignal, For } from "solid-js";
import { createRouteAction, createRouteData, useParams } from "solid-start";
import { Button } from "~/components/Button";
import { Input } from "~/components/Input";
import { ChatsService } from "~/services/chats-service";
import { CommandService } from "~/services/command-service";
import { UserStore } from "~/storage/user-store";

const Message = ({ content, isMe }: { content: string; isMe: boolean }) => {
  return (
    <div class="flex">
      <span
        class=" text-white px-2 mb-1 rounded-full"
        classList={{
          "bg-blue-600 ml-auto": isMe,
          "bg-green-600": !isMe,
        }}
      >
        {content}
      </span>
    </div>
  );
};

const MessageList = ({
  messages,
}: {
  messages: { content: string; isMe: boolean }[];
}) => {
  return (
    <div class="h-full flex flex-col-reverse overflow-y-scroll px-1">
      <For each={messages}>
        {(msg) => <Message content={msg.content} isMe={msg.isMe}></Message>}
      </For>
    </div>
  );
};

export default function ChatPage() {
  const params = useParams<{ chatId: string }>();
  const user = UserStore.get();
  const messages = createRouteData(
    () =>
      ChatsService.getAll(params.chatId).then((it) =>
        it.data
          .map((it) => ({
            content: it.message,
            isMe: user.hash === it.sent_by,
          }))
          .reverse()
      ),
    {
      initialValue: [],
    }
  );
  const [message, setMessage] = createSignal<string>();
  const [_, sendMsg] = createRouteAction(CommandService.sendChatMessage);

  const sendMessage = async () => {
    const msg = message();
    const username = UserStore.get().username;
    if (!username) return;
    if (!msg) return;

    await sendMsg({
      username,
      chat_id: params.chatId,
      message: msg,
    });
    setMessage("");
  };

  return (
    <div class="grid h-screen" style="grid-template-rows: 1fr auto;">
      <MessageList messages={messages() ?? []} />
      <div class="flex w-full mt-2">
        <Input
          value={message()}
          onInput={(it) => setMessage(it)}
          onEnter={sendMessage}
        />
        <Button label="Send!" onClick={() => sendMessage()} />
      </div>
    </div>
  );
}
