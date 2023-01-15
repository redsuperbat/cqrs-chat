// Next.js API route support: https://nextjs.org/docs/api-routes/introduction
import type { NextApiRequest, NextApiResponse } from "next";
import { chatProjectionBaseUrl } from "../url";

export type GetChatsData = {
  chats: { chat_id: string; subject: string }[];
};

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse<GetChatsData>
) {
  const response = await fetch(
    `${chatProjectionBaseUrl}/chats?user_id=${req.query.user_id}`,
    {
      method: req.method,
      headers: {
        "Content-Type": "application/json",
      },
    }
  );
  return res.status(response.status).json(await response.json());
}
