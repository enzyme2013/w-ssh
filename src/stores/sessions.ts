import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type {
  CredentialKind, Session, SessionGroup, CreateSession, UpdateGroup, UpdateSession,
} from '../types'

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

  const groups = computed<SessionGroup[]>(() =>
    Object.entries(groupedSessions.value).map(([name, groupSessions]) => ({
      name,
      icon: groupSessions.find(session => session.group_icon)?.group_icon || 'folder',
      sessions: groupSessions,
      editable: name !== '未分组',
    })),
  )

  function groupIconForName(name?: string) {
    if (!name) return undefined
    return groups.value.find(group => group.name === name)?.icon || 'folder'
  }

  async function updateGroup(data: UpdateGroup) {
    const updated = await invoke<Session[]>('update_group', { data })
    const byId = new Map(updated.map(session => [session.id, session]))
    sessions.value = sessions.value.map(session => byId.get(session.id) || session)
    await consumeNotice()
    return updated
  }

  function clearNotice() {
    notice.value = null
  }

  function replaceSession(updated: Session) {
    const idx = sessions.value.findIndex(session => session.id === updated.id)
    if (idx !== -1) sessions.value[idx] = updated
  }

  async function setCredential(sessionId: string, kind: CredentialKind, secret: string) {
    const updated = await invoke<Session>('set_session_credential', {
      request: { session_id: sessionId, kind, secret },
    })
    replaceSession(updated)
    return updated
  }

  async function confirmCredentialRebind(sessionId: string, kind: CredentialKind) {
    const updated = await invoke<Session>('confirm_session_credential_rebind', {
      sessionId,
      kind,
    })
    replaceSession(updated)
    return updated
  }

  async function migrateLegacyCredential(sessionId: string) {
    const updated = await invoke<Session>('migrate_legacy_credential', { sessionId })
    replaceSession(updated)
    return updated
  }

  return {
    sessions,
    loading,
    notice,
    fetchSessions,
    createSession,
    updateSession,
    deleteSession,
    setCredential,
    confirmCredentialRebind,
    migrateLegacyCredential,
    replaceSession,
    clearNotice,
    groupedSessions,
    groups,
    groupIconForName,
    updateGroup,
  }
})
