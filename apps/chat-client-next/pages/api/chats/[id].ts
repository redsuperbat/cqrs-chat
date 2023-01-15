// Next.js API route support: https://nextjs.org/docs/api-routes/introduction
import type { NextApiRequest, NextApiResponse } from "next";
import { chatProjectionBaseUrl } from "../url";

export type Data = {
  messages: { message: string; sent_by: string; message_id: string }[];
  chat_id: string;
};

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse<Data>
) {
  const response = await fetch(
    `${chatProjectionBaseUrl}/chats/${req.query.id}`,
    {
      method: req.method,
      headers: {
        "Content-Type": "application/json",
      },
    }
  );
  return res.status(response.status).json(await response.json());
}
