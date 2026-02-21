import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Session, CreateSession, UpdateSession } from '../types'

export const useSessionsStore = defineStore('sessions', () => {
  const sessions = ref<Session[]>([])
  const loading = ref(false)

  async function fetchSessions() {
    loading.value = true
    try {
      sessions.value = await invoke<Session[]>('get_sessions')
    } finally {
      loading.value = false
    }
  }

  async function createSession(data: CreateSession) {
    const session = await invoke<Session>('create_session', { data })
    sessions.value.push(session)
    return session
  }

  async function updateSession(data: UpdateSession) {
    const updated = await invoke<Session>('update_session', { data })
    const idx = sessions.value.findIndex(s => s.id === data.id)
    if (idx !== -1) sessions.value[idx] = updated
    return updated
  }

  async function deleteSession(id: string) {
    await invoke('delete_session', { id })
    sessions.value = sessions.value.filter(s => s.id !== id)
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

  return { sessions, loading, fetchSessions, createSession, updateSession, deleteSession, groupedSessions }
})
