<template>
  <div class="flex flex-col items-center justify-center h-full">
    <div class="w-1/4">
      <form class="fieldset bg-base-200 border-base-300 rounded-box border p-4">
        <fieldset class="fieldset w-full" v-if="!loggedIn">
          <label class="label">username</label>
          <input
            v-model="name"
            type="text"
            class="input validator w-full"
            placeholder="Email"
            required
          />
          <p class="validator-hint hidden">Required</p>
        </fieldset>

        <label class="fieldset w-full" v-if="!loggedIn">
          <span class="label">Password</span>
          <input
            v-model="password"
            type="password"
            class="input validator w-full"
            placeholder="Password"
            required
          />
          <span class="validator-hint hidden">Required</span>
        </label>

        <button class="btn btn-primary mt-4" @click.prevent="connect">
          Connect
        </button>
      </form>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { useAuthApi } from "~/composables/api/auth.api";

const appStore = useAppStore();
const router = useRouter();

const loggedIn = ref(false);
const name = ref("");
const password = ref("");

const token = useCookie("token");

const connect = async () => {
  if (!loggedIn.value) {
    token.value = await useAuthApi().login(name.value, password.value);
  }
  router.push("/terminal");
};

watch(
  token,
  async () => {
    if (token.value) {
      let me = await useAuthApi().me();
      if (me) {
        loggedIn.value = true;
      } else {
        loggedIn.value = false;
      }
    } else {
      loggedIn.value = false;
    }
  },
  { immediate: true },
);
</script>

<style></style>
