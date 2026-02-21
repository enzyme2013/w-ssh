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

export interface TerminalTab {
  id: string          // terminal_id (来自后端)
  session_id: string
  session_name: string
  connected: boolean
}
