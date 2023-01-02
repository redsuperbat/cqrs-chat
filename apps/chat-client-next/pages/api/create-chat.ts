// Next.js API route support: https://nextjs.org/docs/api-routes/introduction
import axios from "axios";
import type { NextApiRequest, NextApiResponse } from "next";
import { chatAggregateBaseUrl } from "./url";

type Data = {
  username: string;
};

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse<Data>
) {
  const response = await axios.post(
    `${chatAggregateBaseUrl}/create-chat`,
    req.body
  );
  res.status(200).json(response.data);
}
