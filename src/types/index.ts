export interface Session {
  id: string
  name: string
  host: string
  port: number
  username: string
  password?: string
  private_key?: string
  group_name?: string
  created_at: string
  updated_at: string
}

export interface CreateSession {
  name: string
  host: string
  port: number
  username: string
  password?: string
  private_key?: string
  group_name?: string
}

export type UpdateSession = CreateSession & { id: string }

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
