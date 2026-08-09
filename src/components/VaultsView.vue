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
        v-for="group in groupNavItems"
        :key="group.name"
        class="nav-item"
        :class="{ active: activeNav === group.name }"
        @click="activeNav = group.name"
      >
        <n-icon :component="resolveSessionIcon(group.icon, 'folder')" class="nav-icon" />
        <span class="nav-label">{{ group.name }}</span>
        <n-button
          v-if="group.editable"
          quaternary
          circle
          size="tiny"
          class="group-edit-button"
          :aria-label="`编辑分组 ${group.name}`"
          @click.stop="openGroupEditor(group.name, group.icon)"
        >
          <template #icon><n-icon :component="CreateOutline" /></template>
        </n-button>
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
        <n-tooltip v-if="activeGroup" trigger="hover">
          <template #trigger>
            <n-button
              size="small"
              quaternary
              circle
              :aria-label="`编辑分组 ${activeGroup.name}`"
              @click="openGroupEditor(activeGroup.name, activeGroup.icon)"
            >
              <template #icon><n-icon :component="CreateOutline" /></template>
            </n-button>
          </template>
          编辑当前分组
        </n-tooltip>
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button
              size="small"
              quaternary
              circle
              aria-label="主机信任"
              @click="showTrustSettings = true"
            >
              <template #icon><n-icon :component="ShieldCheckmarkOutline" /></template>
            </n-button>
          </template>
          主机信任
        </n-tooltip>
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button
              size="small"
              quaternary
              circle
              aria-label="存储设置"
              @click="showStorageSettings = true"
            >
              <template #icon><n-icon :component="SettingsOutline" /></template>
            </n-button>
          </template>
          存储设置
        </n-tooltip>
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
            <div class="group-title">
              <n-icon :component="resolveSessionIcon(sessionsStore.groupIconForName(String(group)), 'folder')" />
              <span>{{ group }}</span>
            </div>
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
      :default-group="activeNav !== 'all' && activeNav !== '未分组' ? activeNav : undefined"
      @saved="onSaved"
    />

    <n-modal
      v-model:show="showGroupEditor"
      preset="dialog"
      title="编辑分组"
      :show-icon="false"
    >
      <div class="group-editor">
        <label for="group-name">分组名称</label>
        <n-input
          id="group-name"
          v-model:value="groupDraft.name"
          maxlength="100"
          show-count
          placeholder="输入分组名称"
          @keyup.enter="saveGroup"
        />
        <label>分组图标</label>
        <IconSelect v-model="groupDraft.icon" />
        <p>修改会同步应用到该分组内的全部主机。</p>
      </div>
      <template #action>
        <n-space justify="end">
          <n-button @click="showGroupEditor = false">取消</n-button>
          <n-button
            type="primary"
            :loading="savingGroup"
            :disabled="!groupDraft.name.trim()"
            @click="saveGroup"
          >
            保存分组
          </n-button>
        </n-space>
      </template>
    </n-modal>

    <StorageSettings v-model="showStorageSettings" @changed="handleStorageChanged" />
    <TrustSettings v-model="showTrustSettings" />

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

    <n-modal
      v-model:show="showCredentialPrompt"
      preset="dialog"
      :title="`连接到 ${pendingSession?.name || ''}`"
      :show-icon="false"
      @after-leave="resetCredentialPrompt"
    >
      <n-input
        v-model:value="connectSecret"
        type="password"
        show-password-on="click"
        :placeholder="pendingSession?.auth_method === 'private_key' ? 'Passphrase（未加密私钥可留空）' : '本次连接密码'"
        autofocus
        @keyup.enter="connectWithCredential"
      />
      <n-checkbox v-if="connectSecret" v-model:checked="saveConnectCredential" class="save-credential">
        保存到系统凭据库
      </n-checkbox>
      <template #action>
        <n-space justify="end">
          <n-button @click="showCredentialPrompt = false">取消</n-button>
          <n-button
            type="primary"
            :loading="connecting"
            :disabled="pendingSession?.auth_method === 'password' && !connectSecret"
            @click="connectWithCredential"
          >
            连接
          </n-button>
        </n-space>
      </template>
    </n-modal>

    <n-modal
      v-model:show="showTrustPrompt"
      preset="dialog"
      :title="trustResult?.state === 'changed' ? '主机密钥已变化' : '确认主机指纹'"
      :type="trustResult?.state === 'changed' ? 'error' : 'info'"
      :show-icon="false"
      @after-leave="trustResult = null"
    >
      <n-alert :type="trustResult?.state === 'changed' ? 'error' : 'info'" :show-icon="false">
        {{ trustResult?.state === 'changed'
          ? '已阻止连接。请通过其他可信渠道核对新指纹后再替换。'
          : '这是首次连接，请核对主机提供的 SHA-256 指纹。' }}
      </n-alert>
      <div class="fingerprint-block">
        <span>{{ trustResult?.endpoint }} · {{ trustResult?.algorithm }}</span>
        <code>{{ trustResult?.fingerprint }}</code>
        <template v-if="trustResult?.previous_fingerprints.length">
          <span>原指纹</span>
          <code v-for="fingerprint in trustResult.previous_fingerprints" :key="fingerprint">
            {{ fingerprint }}
          </code>
        </template>
      </div>
      <template #action>
        <n-space justify="end">
          <n-button @click="showTrustPrompt = false">取消</n-button>
          <n-button
            :type="trustResult?.state === 'changed' ? 'error' : 'primary'"
            :loading="trusting"
            @click="confirmHostTrust"
          >
            {{ trustResult?.state === 'changed' ? '我已核对，替换指纹' : '信任并继续' }}
          </n-button>
        </n-space>
      </template>
    </n-modal>

    <n-modal v-model:show="showLegacyPrompt" preset="dialog" title="迁移旧密码" :show-icon="false">
      <n-alert type="warning" :show-icon="false">
        此会话仍有旧版 SQLite 密码。应用不会显示或自动使用它。
      </n-alert>
      <template #action>
        <n-space justify="end">
          <n-button @click="useOneTimeInstead">改用本次输入</n-button>
          <n-button type="primary" :loading="connecting" @click="migrateAndConnect">
            安全迁移并连接
          </n-button>
        </n-space>
      </template>
    </n-modal>

    <n-modal v-model:show="showRebindPrompt" preset="dialog" title="确认凭据重新绑定" :show-icon="false">
      <n-alert type="warning" :show-icon="false">
        主机、端口、用户名或认证方式已变化。确认后才会把已保存凭据用于当前目标。
      </n-alert>
      <template #action>
        <n-space justify="end">
          <n-button @click="showRebindPrompt = false">取消</n-button>
          <n-button type="primary" :loading="connecting" @click="rebindAndConnect">确认并连接</n-button>
        </n-space>
      </template>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import {
  NAlert, NCheckbox, NIcon, NInput, NButton, NModal, NSpace, NTooltip, useMessage,
} from 'naive-ui'
import {
  GlobeOutline,
  KeyOutline,
  SwapHorizontalOutline,
  DocumentTextOutline,
  SearchOutline,
  AddOutline,
  SettingsOutline,
  ShieldCheckmarkOutline,
  CreateOutline,
} from '@vicons/ionicons5'
import { useSessionsStore } from '../stores/sessions'
import { useTerminalsStore } from '../stores/terminals'
import { useStorageStore } from '../stores/storage'
import SessionForm from './SessionForm.vue'
import StorageSettings from './StorageSettings.vue'
import TrustSettings from './TrustSettings.vue'
import HostCard from './HostCard.vue'
import IconSelect from './IconSelect.vue'
import type { CredentialKind, HostTrustResult, Session } from '../types'
import { resolveSessionIcon } from '../utils/sessionIcons'

const message = useMessage()
const sessionsStore = useSessionsStore()
const terminalsStore = useTerminalsStore()
const storageStore = useStorageStore()

// 导航状态
const activeNav = ref<string>('all')
const search = ref('')

// 左侧 Group 列表
const groupNavItems = computed(() => sessionsStore.groups)
const activeGroup = computed(() => sessionsStore.groups.find(
  group => group.editable && group.name === activeNav.value,
))

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
  pendingSession.value = session
  try {
    const result = await invoke<HostTrustResult>('ssh_trust_preflight', { sessionId: session.id })
    if (result.state === 'trusted') {
      await continueAfterTrust(session)
      return
    }
    trustResult.value = result
    showTrustPrompt.value = true
  } catch (e) {
    pendingSession.value = null
    message.error(`主机密钥检查失败: ${e}`)
  }
}

const showCredentialPrompt = ref(false)
const pendingSession = ref<Session | null>(null)
const connectSecret = ref('')
const saveConnectCredential = ref(true)
const connecting = ref(false)
const showTrustPrompt = ref(false)
const trustResult = ref<HostTrustResult | null>(null)
const trusting = ref(false)
const showLegacyPrompt = ref(false)
const showRebindPrompt = ref(false)

function credentialKind(session: Session): CredentialKind {
  return session.auth_method === 'private_key' ? 'private_key_passphrase' : 'password'
}

async function continueAfterTrust(session: Session) {
  pendingSession.value = session
  if (session.credential_state === 'stored') {
    await connectNow()
  } else if (session.credential_state === 'legacy_plaintext') {
    showLegacyPrompt.value = true
  } else if (session.credential_state === 'needs_rebind') {
    showRebindPrompt.value = true
  } else {
    connectSecret.value = ''
    saveConnectCredential.value = true
    showCredentialPrompt.value = true
  }
}

async function confirmHostTrust() {
  if (!trustResult.value?.challenge_id || !pendingSession.value) return
  trusting.value = true
  try {
    const command = trustResult.value.state === 'changed'
      ? 'replace_host_trust'
      : 'accept_host_trust'
    await invoke(command, { challengeId: trustResult.value.challenge_id })
    showTrustPrompt.value = false
    await continueAfterTrust(pendingSession.value)
  } catch (error) {
    message.error(`主机信任未更新: ${error}`)
  } finally {
    trusting.value = false
  }
}

async function connectNow(secret?: string) {
  if (!pendingSession.value) return
  const session = pendingSession.value
  connecting.value = true
  try {
    await terminalsStore.openTerminal(
      session.id,
      session.name,
      120,
      40,
      secret ? { kind: credentialKind(session), secret } : undefined,
    )
    showCredentialPrompt.value = false
    showLegacyPrompt.value = false
    showRebindPrompt.value = false
    pendingSession.value = null
  } catch (e) {
    message.error(`连接失败: ${e}`)
  } finally {
    connecting.value = false
  }
}

async function connectWithCredential() {
  if (!pendingSession.value) return
  const secret = connectSecret.value
  if (secret && saveConnectCredential.value) {
    try {
      pendingSession.value = await sessionsStore.setCredential(
        pendingSession.value.id,
        credentialKind(pendingSession.value),
        secret,
      )
      await connectNow()
      return
    } catch (error) {
      message.warning(`系统凭据库不可用，本次将不保存：${error}`)
    }
  }
  await connectNow(secret || undefined)
}

function resetCredentialPrompt() {
  connectSecret.value = ''
  saveConnectCredential.value = true
  if (!showLegacyPrompt.value && !showRebindPrompt.value) pendingSession.value = null
}

async function migrateAndConnect() {
  if (!pendingSession.value) return
  connecting.value = true
  try {
    pendingSession.value = await sessionsStore.migrateLegacyCredential(pendingSession.value.id)
    showLegacyPrompt.value = false
    connecting.value = false
    await connectNow()
  } catch (error) {
    connecting.value = false
    message.error(`旧密码迁移失败，原数据保持不变: ${error}`)
  }
}

function useOneTimeInstead() {
  showLegacyPrompt.value = false
  connectSecret.value = ''
  saveConnectCredential.value = true
  showCredentialPrompt.value = true
}

async function rebindAndConnect() {
  if (!pendingSession.value) return
  connecting.value = true
  try {
    pendingSession.value = await sessionsStore.confirmCredentialRebind(
      pendingSession.value.id,
      credentialKind(pendingSession.value),
    )
    showRebindPrompt.value = false
    connecting.value = false
    await connectNow()
  } catch (error) {
    connecting.value = false
    message.error(`凭据重新绑定失败: ${error}`)
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

const showGroupEditor = ref(false)
const savingGroup = ref(false)
const editingGroupName = ref('')
const groupDraft = ref({ name: '', icon: 'folder' })

function openGroupEditor(name: string, icon: string) {
  editingGroupName.value = name
  groupDraft.value = { name, icon }
  showGroupEditor.value = true
}

async function saveGroup() {
  const name = groupDraft.value.name.trim()
  if (!name || !editingGroupName.value) return
  savingGroup.value = true
  try {
    const previous = editingGroupName.value
    await sessionsStore.updateGroup({
      current_name: previous,
      name,
      icon: groupDraft.value.icon,
    })
    if (activeNav.value === previous) activeNav.value = name
    showGroupEditor.value = false
    editingGroupName.value = ''
    message.success('分组已更新')
  } catch (error) {
    message.error(`分组更新失败: ${error}`)
  } finally {
    savingGroup.value = false
  }
}

const showStorageSettings = ref(false)
const showTrustSettings = ref(false)

async function handleStorageChanged() {
  try {
    await sessionsStore.fetchSessions()
  } catch (error) {
    message.error(`无法读取会话: ${error}`)
  }
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

watch(
  () => sessionsStore.notice,
  notice => {
    if (!notice) return
    message.warning(notice, { duration: 8000 })
    sessionsStore.clearNotice()
  },
)

onMounted(async () => {
  try {
    await storageStore.fetchStatus()
  } catch (error) {
    message.error(`无法读取存储状态: ${error}`)
  }
  try {
    await sessionsStore.fetchSessions()
  } catch (error) {
    message.error(`无法读取会话: ${error}`)
  }
})
</script>

<style scoped>
.vaults-view {
  display: flex;
  width: 100%;
  height: 100%;
  background: #0d1117;
  color: #e2e8f0;
  overflow: hidden;
}

/* 左侧导航 */
.nav-panel {
  width: 168px;
  flex-shrink: 0;
  background: #090c11;
  border-right: 1px solid #21283a;
  display: flex;
  flex-direction: column;
  padding: 12px 0;
  overflow-y: auto;
  user-select: none;
}

.nav-section-title {
  font-size: 10px;
  font-weight: 600;
  color: #374151;
  text-transform: uppercase;
  letter-spacing: 0.8px;
  padding: 4px 12px 8px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 8px 10px 8px 12px;
  cursor: pointer;
  font-size: 13px;
  color: #8899aa;
  border-radius: 6px;
  margin: 1px 6px;
  transition: background 0.15s, color 0.15s, box-shadow 0.15s;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.nav-item:hover {
  background: rgba(255, 255, 255, 0.04);
  color: #c9d5e0;
}

.nav-item.active {
  background: rgba(107, 156, 248, 0.10);
  color: #6b9cf8;
  box-shadow: inset 3px 0 0 0 #6b9cf8;
}

.nav-label {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
}

.group-edit-button {
  margin-inline-start: auto;
  flex-shrink: 0;
  opacity: 0;
}

.nav-item:hover .group-edit-button,
.nav-item:focus-within .group-edit-button,
.nav-item.active .group-edit-button {
  opacity: 1;
}

.nav-item.placeholder {
  color: #374151;
}

.nav-item.placeholder:hover {
  color: #4a5568;
}

.nav-icon {
  flex-shrink: 0;
  font-size: 15px;
}

.nav-divider {
  height: 1px;
  background: #21283a;
  margin: 10px 14px;
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
  padding: 14px 20px;
  border-bottom: 1px solid #21283a;
  background: #0d1117;
  flex-shrink: 0;
}

.search-input {
  flex: 1;
}

.cards-scroll {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
  background: #0d1117;
}

.group-title {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 10px;
  font-weight: 600;
  color: #374151;
  text-transform: uppercase;
  letter-spacing: 0.8px;
  margin-bottom: 12px;
  margin-top: 4px;
}

.group-title:not(:first-child) {
  margin-top: 20px;
}

.cards-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(156px, 1fr));
  gap: 14px;
}

.empty-hint {
  text-align: center;
  color: #4a5568;
  font-size: 13px;
  padding: 48px 16px;
}

.group-editor {
  display: grid;
  gap: 10px;
}

.group-editor label {
  color: #c9d5e0;
  font-size: 13px;
  font-weight: 600;
}

.group-editor label:not(:first-child) {
  margin-top: 6px;
}

.group-editor p {
  color: #8899aa;
  font-size: 12px;
  line-height: 1.5;
}

.save-credential {
  margin-top: 14px;
}

.fingerprint-block {
  display: grid;
  gap: 8px;
  margin-top: 16px;
}

.fingerprint-block span {
  color: #8899aa;
  font-size: 12px;
}

.fingerprint-block code {
  color: #e2e8f0;
  font-size: 12px;
  overflow-wrap: anywhere;
}

@media (max-width: 520px) {
  .nav-panel {
    width: 52px;
  }

  .nav-section-title,
  .nav-item > span,
  .group-edit-button {
    display: none;
  }

  .nav-item {
    justify-content: center;
    padding: 8px;
  }

  .nav-divider {
    margin-inline: 10px;
  }

  .content-header {
    padding-inline: 12px;
  }

  .search-input {
    min-width: 0;
  }

  .cards-scroll {
    padding: 16px 12px;
  }
}
</style>
