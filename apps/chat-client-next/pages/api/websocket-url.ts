// Next.js API route support: https://nextjs.org/docs/api-routes/introduction
import type { NextApiRequest, NextApiResponse } from "next";
import { chatProjectionWebsocketUrl } from "./url";

export type GetWebsocketUrlData = {
  url: string;
};

export default function handler(
  req: NextApiRequest,
  res: NextApiResponse<GetWebsocketUrlData>
) {
  if (req.method !== "GET") {
    return res.status(400);
  }
  if (!chatProjectionWebsocketUrl) {
    return res.status(500);
  }
  res.status(200).json({ url: chatProjectionWebsocketUrl });
}
