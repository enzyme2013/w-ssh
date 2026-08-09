export type AuthMethod = 'password' | 'private_key'
export type CredentialKind = 'password' | 'private_key_passphrase'
export type CredentialState = 'none' | 'stored' | 'legacy_plaintext' | 'needs_rebind'

export interface Session {
  id: string
  name: string
  host: string
  port: number
  username: string
  private_key?: string
  auth_method: AuthMethod
  credential_state: CredentialState
  icon?: string
  group_name?: string
  group_icon?: string
  created_at: string
  updated_at: string
}

export interface CreateSession {
  name: string
  host: string
  port: number
  username: string
  private_key?: string
  auth_method: AuthMethod
  icon?: string
  group_name?: string
  group_icon?: string
}

export type UpdateSession = CreateSession & { id: string }

export interface UpdateGroup {
  current_name: string
  name: string
  icon?: string
}

export interface SessionGroup {
  name: string
  icon: string
  sessions: Session[]
  editable: boolean
}

export interface ConnectSecret {
  kind: CredentialKind
  secret: string
}

export interface HostTrustResult {
  state: 'trusted' | 'unknown' | 'changed'
  endpoint: string
  algorithm: string
  fingerprint: string
  previous_fingerprints: string[]
  challenge_id?: string
}

export interface HostTrustEntry {
  endpoint: string
  algorithm: string
  fingerprint: string
}

export interface LegacyCredentialSummary {
  count: number
  session_ids: string[]
}

export type StorageBackend = 'sqlite' | 'yaml'

export interface StorageSelection {
  backend: StorageBackend
  yaml_path?: string
}

export interface StorageStatus {
  backend: StorageBackend
  yaml_path?: string
  sqlite_path: string
  active_error?: string
}

export interface StorageCopyResult {
  copied: number
  status: StorageStatus
}

export interface TerminalTab {
  id: string          // terminal_id (来自后端)
  session_id: string
  session_name: string
  connected: boolean
}
