<template>
  <div class="vaults-view">
    <!-- 左侧导航 -->
    <div class="nav-panel">
      <div class="nav-section-title">Hosts</div>

      <div
        class="nav-item"
        :class="{ active: activeNav === 'all' }"
        @click="activeNav = 'all'"
      >
        <n-icon :component="GlobeOutline" class="nav-icon" />
        <span>All</span>
      </div>

      <div
        v-for="group in groupNames"
        :key="group"
        class="nav-item"
        :class="{ active: activeNav === group }"
        @click="activeNav = group"
      >
        <n-icon :component="FolderOutline" class="nav-icon" />
        <span>{{ group }}</span>
      </div>

      <div class="nav-divider" />

      <div class="nav-item placeholder" @click="message.info('功能开发中')">
        <n-icon :component="KeyOutline" class="nav-icon" />
        <span>Keys</span>
      </div>
      <div class="nav-item placeholder" @click="message.info('功能开发中')">
        <n-icon :component="SwapHorizontalOutline" class="nav-icon" />
        <span>Port Fwd</span>
      </div>
      <div class="nav-item placeholder" @click="message.info('功能开发中')">
        <n-icon :component="DocumentTextOutline" class="nav-icon" />
        <span>Logs</span>
      </div>
    </div>

    <!-- 右侧内容 -->
    <div class="content-panel">
      <!-- 顶部操作栏 -->
      <div class="content-header">
        <n-input
          v-model:value="search"
          size="small"
          placeholder="搜索主机..."
          clearable
          class="search-input"
        >
          <template #prefix><n-icon :component="SearchOutline" /></template>
        </n-input>
        <n-button size="small" type="primary" @click="handleNewHost">
          <template #icon><n-icon :component="AddOutline" /></template>
          New Host
        </n-button>
      </div>

      <!-- 卡片区域 -->
      <div class="cards-scroll">
        <!-- 全部模式：按 Group 分组展示 -->
        <template v-if="activeNav === 'all'">
          <template v-for="(items, group) in filteredGroupedSessions" :key="group">
            <div class="group-title">{{ group }}</div>
            <div class="cards-grid">
              <HostCard
                v-for="session in items"
                :key="session.id"
                :session="session"
                @connect="handleConnect"
                @edit="handleEdit"
                @delete="handleDeleteRequest"
              />
            </div>
          </template>
          <div v-if="Object.keys(filteredGroupedSessions).length === 0" class="empty-hint">
            暂无主机，点击 New Host 添加
          </div>
        </template>

        <!-- 单组模式：只显示该 Group 卡片 -->
        <template v-else>
          <div v-if="currentGroupSessions.length > 0" class="cards-grid">
            <HostCard
              v-for="session in currentGroupSessions"
              :key="session.id"
              :session="session"
              @connect="handleConnect"
              @edit="handleEdit"
              @delete="handleDeleteRequest"
            />
          </div>
          <div v-else class="empty-hint">该分组暂无主机</div>
        </template>
      </div>
    </div>

    <!-- 新建/编辑表单 -->
    <SessionForm
      v-model="showForm"
      :session="editingSession"
      @saved="onSaved"
    />

    <!-- 删除确认 -->
    <n-modal v-model:show="showDeleteConfirm" preset="dialog" title="确认删除" type="warning">
      <span>删除主机「{{ deletingSession?.name }}」？此操作不可撤销。</span>
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
import { ref, computed, onMounted } from 'vue'
import {
  NIcon, NInput, NButton, NModal, NSpace, useMessage,
} from 'naive-ui'
import {
  GlobeOutline,
  FolderOutline,
  KeyOutline,
  SwapHorizontalOutline,
  DocumentTextOutline,
  SearchOutline,
  AddOutline,
} from '@vicons/ionicons5'
import { useSessionsStore } from '../stores/sessions'
import { useTerminalsStore } from '../stores/terminals'
import SessionForm from './SessionForm.vue'
import HostCard from './HostCard.vue'
import type { Session } from '../types'

const message = useMessage()
const sessionsStore = useSessionsStore()
const terminalsStore = useTerminalsStore()

// 导航状态
const activeNav = ref<string>('all')
const search = ref('')

// 左侧 Group 列表
const groupNames = computed(() => Object.keys(sessionsStore.groupedSessions))

// 全部模式：按 Group 过滤+搜索
const filteredGroupedSessions = computed(() => {
  const q = search.value.toLowerCase()
  const result: Record<string, Session[]> = {}
  for (const [group, items] of Object.entries(sessionsStore.groupedSessions)) {
    const matched = q
      ? items.filter(s =>
          s.name.toLowerCase().includes(q) || s.host.toLowerCase().includes(q))
      : items
    if (matched.length) result[group] = matched
  }
  return result
})

// 单组模式：当前组的会话列表
const currentGroupSessions = computed(() => {
  if (activeNav.value === 'all') return []
  const items = sessionsStore.groupedSessions[activeNav.value] ?? []
  const q = search.value.toLowerCase()
  return q
    ? items.filter(s =>
        s.name.toLowerCase().includes(q) || s.host.toLowerCase().includes(q))
    : items
})

// 连接
async function handleConnect(session: Session) {
  try {
    await terminalsStore.openTerminal(session.id, session.name, 120, 40)
  } catch (e) {
    message.error(`连接失败: ${e}`)
  }
}

// 新建/编辑
const showForm = ref(false)
const editingSession = ref<Session | undefined>()

function handleNewHost() {
  editingSession.value = undefined
  showForm.value = true
}

function handleEdit(session: Session) {
  editingSession.value = session
  showForm.value = true
}

function onSaved() {
  editingSession.value = undefined
}

// 删除
const showDeleteConfirm = ref(false)
const deletingSession = ref<Session | null>(null)
const deleting = ref(false)

function handleDeleteRequest(session: Session) {
  deletingSession.value = session
  showDeleteConfirm.value = true
}

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
.vaults-view {
  display: flex;
  width: 100%;
  height: 100%;
  background: #1e1e2e;
  color: #cdd6f4;
  overflow: hidden;
}

/* 左侧导航 */
.nav-panel {
  width: 160px;
  flex-shrink: 0;
  background: #181825;
  border-right: 1px solid #313244;
  display: flex;
  flex-direction: column;
  padding: 8px 0;
  overflow-y: auto;
  user-select: none;
}

.nav-section-title {
  font-size: 11px;
  font-weight: 600;
  color: #6c7086;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  padding: 4px 12px 8px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 12px;
  cursor: pointer;
  font-size: 13px;
  color: #a6adc8;
  border-radius: 4px;
  margin: 0 4px;
  transition: background 0.15s, color 0.15s;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.nav-item:hover {
  background: #313244;
  color: #cdd6f4;
}

.nav-item.active {
  background: #313244;
  color: #89b4fa;
}

.nav-item.placeholder {
  color: #585b70;
}

.nav-item.placeholder:hover {
  color: #6c7086;
}

.nav-icon {
  flex-shrink: 0;
  font-size: 15px;
}

.nav-divider {
  height: 1px;
  background: #313244;
  margin: 8px 12px;
}

/* 右侧内容 */
.content-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  overflow: hidden;
}

.content-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  border-bottom: 1px solid #313244;
  flex-shrink: 0;
}

.search-input {
  flex: 1;
}

.cards-scroll {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
}

.group-title {
  font-size: 11px;
  font-weight: 600;
  color: #6c7086;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 10px;
  margin-top: 4px;
}

.group-title:not(:first-child) {
  margin-top: 20px;
}

.cards-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
  gap: 12px;
}

.empty-hint {
  text-align: center;
  color: #6c7086;
  font-size: 13px;
  padding: 48px 16px;
}
</style>
