// Next.js API route support: https://nextjs.org/docs/api-routes/introduction
import type { NextApiRequest, NextApiResponse } from "next";
import { chatAggregateBaseUrl } from "./url";

type Data = {
  username: string;
};

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse<Data>
) {
  const response = await fetch(`${chatAggregateBaseUrl}/create-chat`, {
    body: JSON.stringify(req.body),
    method: req.method,
    headers: {
      "Content-Type": "application/json",
    },
  });
  return res.status(response.status).json(await response.json());
}
