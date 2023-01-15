// Next.js API route support: https://nextjs.org/docs/api-routes/introduction
import axios, { isAxiosError } from "axios";
import type { NextApiRequest, NextApiResponse } from "next";
import { chatAggregateBaseUrl } from "./url";

type Data = {
  username: string;
};

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse<Data>
) {
  if (req.method !== "POST") {
    return res.status(400);
  }

  try {
    const response = await axios.post(
      `${chatAggregateBaseUrl}/create-chat`,
      req.body
    );
    res.status(200).json(response.data);
  } catch (e) {
    if (isAxiosError(e) && e.response) {
      return res.status(e.response.status).json(e.response.data);
    }
    return res.status(500);
  }
}
