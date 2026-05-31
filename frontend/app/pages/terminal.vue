<template>
  <div class="flex items-center justify-center h-full">
    <div
      class="w-3/4 h-3/4 rounded-xl overflow-hidden outline-2 outline-red-500 bg-black p-2"
    >
      <div id="terminal" ref="terminalRef" class="w-full h-full"></div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { AttachAddon } from "@xterm/addon-attach";
import { onMounted, ref } from "vue";
const appStore = useAppStore();
const term = ref<Terminal>();
const ROWS = 50;
const COLS = 200;
const terminalRef = ref<HTMLDivElement>();
let socket = ref<WebSocket>();
const initXterm = () => {
  term.value = new Terminal({
    letterSpacing: 0,
    allowProposedApi: true,
    rows: ROWS,
    cols: COLS,
    convertEol: true,
    scrollback: 50,
    // disableStdin: true,
    tabStopWidth: 4,
    // cursorStyle: "underline",
    cursorBlink: true,
    lineHeight: 1.2,
    fontSize: 16,
    screenReaderMode: true,
    fontFamily: "JetBrainsMonoNFM, monospace",
    theme: {
      foreground: "#ECECEC",
      background: "#000000",
      cursor: "help",
    },
  });
  const fitAddon = new FitAddon();
  term.value.loadAddon(fitAddon);
  if (!terminalRef.value) {
    return;
  }
  term.value.open(terminalRef.value);
  fitAddon.fit();

  // window.addEventListener("resize", resizeScreen);
  // function resizeScreen() {
  //   try {
  //     fitAddon.fit();
  //   } catch (e) {
  //     console.log("e", e.message);
  //   }
  // }
  setTimeout(async () => {
    socket.value = new WebSocket("/api/terminal/ws");
    runFakeTerminal();
    socket.value.onopen = runRealTerminal;
    socket.value.onclose = runFakeTerminal;
    socket.value.onerror = runFakeTerminal;
  }, 1000);
};
function runRealTerminal() {
  if (!socket.value) {
    return;
  }
  const attachAddon = new AttachAddon(socket.value);
  term.value?.loadAddon(attachAddon);
  term.value?.focus();
}
const runFakeTerminal = () => {
  if (!term.value) {
    return;
  }
  //@ts-expect-error
  if (term.value._initialized) return;
  //@ts-expect-error
  term.value._initialized = true;
  term.value.writeln("");
  term.value.writeln("############################");
  term.value.writeln("#### Waiting to connect ####");
  term.value.writeln("############################");

  term.value.onKey((e: { domEvent: any }) => {
    if (!term.value) {
      return;
    }
    const ev = e.domEvent;
    // socket.send(ev.code);
    const printable = !ev.altKey && !ev.ctrlKey && !ev.metaKey;
    if (ev.code === "Enter") {
      // term.value.prompt();
    } else if (ev.code === "Backspace") {
      //@ts-ignore
      if (term.value._core.buffer.x > 2) {
        // term.value.write("\b \b");
      }
    } else if (printable) {
      // term.value.write(e.key);
    }
  });
  term.value.onData((key) => {
    console.log(key, "onData");
    // if (key.length > 1) term.value.write(key)
  });
};
onMounted(() => {
  if (terminalRef.value) {
    console.log(terminalRef.value);
    initXterm();
  }
});
</script>
<style>
.terminal {
  height: 100%;
}
</style>
