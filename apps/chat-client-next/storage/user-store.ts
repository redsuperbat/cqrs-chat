export type UserStoreState = {
  username: string;
  hashedUsername: string;
};

const key = "user-store";

export const UserStore = {
  get() {
    if (typeof window === "undefined") return;
    const json = localStorage.getItem(key);
    if (!json) return;
    return JSON.parse(json) as UserStoreState;
  },
  set(state: UserStoreState) {
    if (typeof window === "undefined") return;
    localStorage.setItem(key, JSON.stringify(state));
  },
};
