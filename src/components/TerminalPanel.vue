<template>
  <div ref="containerRef" class="terminal-container" />
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch } from 'vue'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { WebLinksAddon } from '@xterm/addon-web-links'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import '@xterm/xterm/css/xterm.css'

const props = defineProps<{
  terminalId: string
  active: boolean
}>()

const emit = defineEmits<{
  closed: []
}>()

const containerRef = ref<HTMLElement | null>(null)
let terminal: Terminal | null = null
let fitAddon: FitAddon | null = null
let unlistenData: UnlistenFn | null = null
let unlistenClosed: UnlistenFn | null = null
let resizeObserver: ResizeObserver | null = null

onMounted(async () => {
  if (!containerRef.value) return

  terminal = new Terminal({
    theme: {
      background: '#1e1e2e',
      foreground: '#cdd6f4',
      cursor: '#f5e0dc',
      selectionBackground: '#45475a',
      black: '#45475a',
      red: '#f38ba8',
      green: '#a6e3a1',
      yellow: '#f9e2af',
      blue: '#89b4fa',
      magenta: '#f5c2e7',
      cyan: '#94e2d5',
      white: '#bac2de',
      brightBlack: '#585b70',
      brightRed: '#f38ba8',
      brightGreen: '#a6e3a1',
      brightYellow: '#f9e2af',
      brightBlue: '#89b4fa',
      brightMagenta: '#f5c2e7',
      brightCyan: '#94e2d5',
      brightWhite: '#a6adc8',
    },
    fontFamily: '"Cascadia Code", "JetBrains Mono", "Fira Code", Consolas, monospace',
    fontSize: 14,
    lineHeight: 1.2,
    cursorBlink: true,
    cursorStyle: 'block',
    allowProposedApi: true,
  })

  fitAddon = new FitAddon()
  terminal.loadAddon(fitAddon)
  terminal.loadAddon(new WebLinksAddon())
  terminal.open(containerRef.value)
  fitAddon.fit()

  // 用户输入 → SSH
  terminal.onData((data) => {
    const bytes = Array.from(new TextEncoder().encode(data))
    invoke('ssh_write', { terminalId: props.terminalId, data: bytes }).catch(console.error)
  })

  // 窗口大小 → SSH resize
  terminal.onResize(({ cols, rows }) => {
    invoke('ssh_resize', { terminalId: props.terminalId, cols, rows }).catch(console.error)
  })

  // SSH 数据 → 终端渲染
  unlistenData = await listen<number[]>(`ssh_data_${props.terminalId}`, (event) => {
    terminal?.write(new Uint8Array(event.payload))
  })

  // SSH 连接关闭
  unlistenClosed = await listen(`ssh_closed_${props.terminalId}`, () => {
    terminal?.writeln('\r\n\x1b[33m--- 连接已断开 ---\x1b[0m')
    emit('closed')
  })

  // 响应容器大小变化
  resizeObserver = new ResizeObserver(() => {
    if (props.active) fitAddon?.fit()
  })
  resizeObserver.observe(containerRef.value)
})

// 切换标签时重新 fit
watch(
  () => props.active,
  (active) => {
    if (active) {
      setTimeout(() => fitAddon?.fit(), 50)
    }
  },
)

onBeforeUnmount(() => {
  unlistenData?.()
  unlistenClosed?.()
  resizeObserver?.disconnect()
  terminal?.dispose()
})
</script>

<style scoped>
.terminal-container {
  width: 100%;
  height: 100%;
  padding: 4px;
  box-sizing: border-box;
  background: #1e1e2e;
}
</style>
