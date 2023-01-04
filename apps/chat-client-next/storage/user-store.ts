export type UserStoreState = {
  username: string;
  hashedUsername: string;
};

const key = "user-store";
const defaultUser = {
  username: "Default user",
  hashedUsername:
    "942b74ac58192ee4c82f03e67064c52bf6baa0dc88663e5f801d3d16daf70ff1",
};

export const UserStore = {
  get() {
    const json = localStorage.getItem(key);
    if (!json) {
      return defaultUser;
    }
    return JSON.parse(json) as UserStoreState;
  },
  set(state: UserStoreState) {
    localStorage.setItem(key, JSON.stringify(state));
  },
};
