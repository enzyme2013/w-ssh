import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Session, CreateSession, UpdateSession } from '../types'

export const useSessionsStore = defineStore('sessions', () => {
  const sessions = ref<Session[]>([])
  const loading = ref(false)
  const notice = ref<string | null>(null)

  async function consumeNotice() {
    try {
      notice.value = await invoke<string | null>('take_storage_notice')
    } catch {
      // Notice retrieval must not turn a completed CRUD operation into a false failure.
    }
  }

  async function fetchSessions() {
    loading.value = true
    try {
      sessions.value = await invoke<Session[]>('get_sessions')
      await consumeNotice()
    } finally {
      loading.value = false
    }
  }

  async function createSession(data: CreateSession) {
    const session = await invoke<Session>('create_session', { data })
    sessions.value.push(session)
    await consumeNotice()
    return session
  }

  async function updateSession(data: UpdateSession) {
    const updated = await invoke<Session>('update_session', { data })
    const idx = sessions.value.findIndex(s => s.id === data.id)
    if (idx !== -1) sessions.value[idx] = updated
    await consumeNotice()
    return updated
  }

  async function deleteSession(id: string) {
    await invoke('delete_session', { id })
    sessions.value = sessions.value.filter(s => s.id !== id)
    await consumeNotice()
  }

  const groupedSessions = computed(() => {
    const groups: Record<string, Session[]> = {}
    for (const s of sessions.value) {
      const key = s.group_name || '未分组'
      if (!groups[key]) groups[key] = []
      groups[key].push(s)
    }
    return groups
  })

  function clearNotice() {
    notice.value = null
  }

  return {
    sessions,
    loading,
    notice,
    fetchSessions,
    createSession,
    updateSession,
    deleteSession,
    clearNotice,
    groupedSessions,
  }
})
