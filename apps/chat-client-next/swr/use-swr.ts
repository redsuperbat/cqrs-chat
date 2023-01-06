import axios from "axios";
import swr from "swr";

const fetcher = (url: string) =>
  axios
    .get(url)
    .then((it) => it.data)
    .catch((e) => {
      throw e.response.data;
    });

export const useSwr = <T = unknown>(url: string) => swr<T>(url, fetcher);
