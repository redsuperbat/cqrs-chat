export type UserState = { username?: string; hash?: string };

const key = "user-store";

export const UserStore = {
  get: () => JSON.parse(localStorage.getItem(key) ?? "{}") as UserState,
  set: (state: UserState) => {
    localStorage.setItem(key, JSON.stringify(state));
  },
};
