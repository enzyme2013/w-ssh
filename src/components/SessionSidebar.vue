<template>
  <div class="sidebar">
    <!-- 顶部操作栏 -->
    <div class="sidebar-header">
      <span class="sidebar-title">会话</span>
      <n-button size="small" quaternary circle @click="showCreate = true">
        <template #icon><n-icon :component="AddOutline" /></template>
      </n-button>
    </div>

    <!-- 搜索框 -->
    <div class="sidebar-search">
      <n-input
        v-model:value="search"
        size="small"
        placeholder="搜索会话..."
        clearable
      >
        <template #prefix><n-icon :component="SearchOutline" /></template>
      </n-input>
    </div>

    <!-- 会话列表 -->
    <div class="sidebar-list">
      <template v-for="(items, group) in filteredGroups" :key="group">
        <div class="group-label">{{ group }}</div>
        <div
          v-for="session in items"
          :key="session.id"
          class="session-item"
          @dblclick="handleConnect(session)"
          @contextmenu.prevent="showContextMenu($event, session)"
        >
          <n-icon :component="ServerOutline" class="session-icon" />
          <div class="session-info">
            <div class="session-name">{{ session.name }}</div>
            <div class="session-host">{{ session.username }}@{{ session.host }}:{{ session.port }}</div>
          </div>
        </div>
      </template>

      <div v-if="Object.keys(filteredGroups).length === 0" class="empty-hint">
        暂无会话，点击 + 新建
      </div>
    </div>

    <!-- 右键菜单 -->
    <n-dropdown
      trigger="manual"
      :x="ctxX"
      :y="ctxY"
      :options="ctxOptions"
      :show="ctxVisible"
      @select="handleCtxSelect"
      @clickoutside="ctxVisible = false"
    />

    <!-- 新建/编辑表单 -->
    <SessionForm
      v-model="showCreate"
      :session="editingSession"
      @saved="onSaved"
    />

    <!-- 删除确认 -->
    <n-modal v-model:show="showDeleteConfirm" preset="dialog" title="确认删除" type="warning">
      <span>删除会话「{{ deletingSession?.name }}」？此操作不可撤销。</span>
      <template #action>
        <n-space justify="end">
          <n-button @click="showDeleteConfirm = false">取消</n-button>
          <n-button type="error" :loading="deleting" @click="confirmDelete">删除</n-button>
        </n-space>
      </template>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { h, ref, computed, onMounted } from 'vue'
import {
  NButton, NIcon, NInput, NDropdown, NModal, NSpace, useMessage,
  type DropdownOption,
} from 'naive-ui'
import { AddOutline, SearchOutline, ServerOutline, CreateOutline, TrashOutline } from '@vicons/ionicons5'
import { useSessionsStore } from '../stores/sessions'
import { useTerminalsStore } from '../stores/terminals'
import SessionForm from './SessionForm.vue'
import type { Session } from '../types'

const message = useMessage()
const sessionsStore = useSessionsStore()
const terminalsStore = useTerminalsStore()

// 搜索
const search = ref('')
const filteredGroups = computed(() => {
  const q = search.value.toLowerCase()
  const result: Record<string, Session[]> = {}
  for (const [group, items] of Object.entries(sessionsStore.groupedSessions)) {
    const matched = q
      ? items.filter(s => s.name.toLowerCase().includes(q) || s.host.toLowerCase().includes(q))
      : items
    if (matched.length) result[group] = matched
  }
  return result
})

// 新建/编辑
const showCreate = ref(false)
const editingSession = ref<Session | undefined>()
function onSaved() {
  editingSession.value = undefined
}

// 右键菜单
const ctxVisible = ref(false)
const ctxX = ref(0)
const ctxY = ref(0)
const ctxSession = ref<Session | null>(null)
const ctxOptions: DropdownOption[] = [
  { label: '连接', key: 'connect', icon: () => h(NIcon, null, { default: () => h(ServerOutline) }) },
  { label: '编辑', key: 'edit', icon: () => h(NIcon, null, { default: () => h(CreateOutline) }) },
  { type: 'divider', key: 'd' },
  { label: '删除', key: 'delete', icon: () => h(NIcon, null, { default: () => h(TrashOutline) }) },
]

function showContextMenu(e: MouseEvent, session: Session) {
  ctxSession.value = session
  ctxX.value = e.clientX
  ctxY.value = e.clientY
  ctxVisible.value = true
}

function handleCtxSelect(key: string) {
  ctxVisible.value = false
  if (!ctxSession.value) return
  if (key === 'connect') handleConnect(ctxSession.value)
  else if (key === 'edit') { editingSession.value = ctxSession.value; showCreate.value = true }
  else if (key === 'delete') { deletingSession.value = ctxSession.value; showDeleteConfirm.value = true }
}

// 连接
async function handleConnect(session: Session) {
  try {
    await terminalsStore.openTerminal(session.id, session.name, 120, 40)
  } catch (e) {
    message.error(`连接失败: ${e}`)
  }
}

// 删除
const showDeleteConfirm = ref(false)
const deletingSession = ref<Session | null>(null)
const deleting = ref(false)

async function confirmDelete() {
  if (!deletingSession.value) return
  deleting.value = true
  try {
    await sessionsStore.deleteSession(deletingSession.value.id)
    message.success('已删除')
    showDeleteConfirm.value = false
  } catch (e) {
    message.error(String(e))
  } finally {
    deleting.value = false
  }
}

onMounted(() => sessionsStore.fetchSessions())
</script>

<style scoped>
.sidebar {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #1e1e2e;
  color: #cdd6f4;
  user-select: none;
}

.sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 12px 8px;
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.5px;
  text-transform: uppercase;
  color: #6c7086;
}

.sidebar-search {
  padding: 0 8px 8px;
}

.sidebar-list {
  flex: 1;
  overflow-y: auto;
  padding: 4px 0;
}

.group-label {
  font-size: 11px;
  color: #6c7086;
  padding: 6px 12px 2px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.session-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 12px;
  cursor: pointer;
  border-radius: 4px;
  margin: 0 4px;
  transition: background 0.15s;
}

.session-item:hover {
  background: #313244;
}

.session-icon {
  font-size: 16px;
  color: #89b4fa;
  flex-shrink: 0;
}

.session-info {
  min-width: 0;
}

.session-name {
  font-size: 13px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.session-host {
  font-size: 11px;
  color: #6c7086;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.empty-hint {
  text-align: center;
  color: #6c7086;
  font-size: 13px;
  padding: 32px 16px;
}
</style>
