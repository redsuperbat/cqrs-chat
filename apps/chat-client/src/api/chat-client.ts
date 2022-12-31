import axios from "axios";

export const ChatClient = axios.create({
  baseURL: "http://localhost:8080",
});
