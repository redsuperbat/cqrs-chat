// Next.js API route support: https://nextjs.org/docs/api-routes/introduction
import axios from "axios";
import type { NextApiRequest, NextApiResponse } from "next";
import { chatProjectionBaseUrl } from "../url";

export type Data = {
  messages: { message: string; sent_by: string; message_id: string }[];
  chat_id: string;
};

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse<Data | undefined>
) {
  const { id } = req.query;
  if (id === "undefined") {
    return res.status(400).send(undefined);
  }

  const { data } = await axios.get(`${chatProjectionBaseUrl}/chats/${id}`);
  res.status(200).json(data);
}
