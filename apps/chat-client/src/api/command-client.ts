import axios from "axios";

export const CommandClient = axios.create({ baseURL: "http://localhost:8081" });
