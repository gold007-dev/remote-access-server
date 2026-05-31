import { defineStore } from "pinia";

export const useAppStore = defineStore("app", () => {
  const apiBase = ref("localhost:1234");
  return {
    apiBase,
  };
});
