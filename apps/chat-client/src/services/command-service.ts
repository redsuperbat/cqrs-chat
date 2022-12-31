import { CommandClient } from "../api/command-client";

type BaseResponse<T> = {
  data: T;
  status: number;
  message: string;
};

type SendChatMessageDto = {
  chat_id: String;
  message: String;
  username: String;
};

export const CommandService = {
  createChat: (username: string) =>
    CommandClient.post<BaseResponse<{ chat_id: string; user_id: string }>>(
      "/create-chat",
      {
        username,
      }
    ),
  sendChatMessage: (dto: SendChatMessageDto) =>
    CommandClient.post("/send-chat-message", dto),
};
