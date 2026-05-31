export const useAuthApi = () => {
  const appStore = useAppStore();
  const me = () => {
    return $fetch("/api/auth/me");
  };
  const login = (name: string, password: string) => {
    return $fetch<string>("/api/auth/login", {
      method: "POST",
      body: JSON.stringify({
        name: name,
        password: password,
      }),
    });
  };
  return { me, login };
};
