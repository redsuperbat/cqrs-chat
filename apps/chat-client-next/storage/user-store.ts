export type UserStoreState = {
  username: string;
  hashedUsername: string;
};

const key = "user-store";

export const UserStore = {
  get() {
    return JSON.parse(localStorage.getItem(key) ?? "{}") as UserStoreState;
  },
  set(state: UserStoreState) {
    localStorage.setItem(key, JSON.stringify(state));
  },
};
