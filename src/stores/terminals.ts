import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { TerminalTab } from '../types'

export const useTerminalsStore = defineStore('terminals', () => {
  const tabs = ref<TerminalTab[]>([])
  const activeTabId = ref<string | null>(null)

  async function openTerminal(sessionId: string, sessionName: string, cols: number, rows: number) {
    const terminalId = await invoke<string>('ssh_connect', {
      sessionId,
      cols,
      rows,
    })

    const tab: TerminalTab = {
      id: terminalId,
      session_id: sessionId,
      session_name: sessionName,
      connected: true,
    }

    tabs.value.push(tab)
    activeTabId.value = terminalId
    return terminalId
  }

  async function closeTab(terminalId: string) {
    await invoke('ssh_disconnect', { terminalId })
    const idx = tabs.value.findIndex(t => t.id === terminalId)
    tabs.value.splice(idx, 1)
    if (activeTabId.value === terminalId) {
      activeTabId.value = tabs.value[tabs.value.length - 1]?.id ?? null
    }
  }

  function markDisconnected(terminalId: string) {
    const tab = tabs.value.find(t => t.id === terminalId)
    if (tab) tab.connected = false
  }

  return { tabs, activeTabId, openTerminal, closeTab, markDisconnected }
})
