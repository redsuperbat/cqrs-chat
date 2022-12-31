import { ChatClient } from "../api/chat-client";

type GetAllChatsDto = {
  message: string;
  sent_by: string;
};

export const ChatsService = {
  getAll: (chat_id: string) =>
    ChatClient.get<GetAllChatsDto[]>(`/chats/${chat_id}`),
};
