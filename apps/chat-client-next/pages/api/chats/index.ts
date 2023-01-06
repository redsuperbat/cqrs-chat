// Next.js API route support: https://nextjs.org/docs/api-routes/introduction
import axios from "axios";
import type { NextApiRequest, NextApiResponse } from "next";
import { chatProjectionBaseUrl } from "../url";

export type GetChatsData = {
  chats: { chat_id: string; subject: string }[];
};

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse<GetChatsData>
) {
  if (req.method !== "GET") {
    return res.status(400);
  }

  const { data } = await axios
    .get(`${chatProjectionBaseUrl}/chats?user_id=${req.query.user_id}`)
    .catch((e) => {
      throw e.message;
    });
  res.status(200).json(data);
}
