<template>
  <n-modal
    v-model:show="visible"
    preset="card"
    title="存储设置"
    class="storage-modal"
    :bordered="false"
    @after-leave="resetDraft"
  >
    <n-spin :show="store.loading">
      <div class="settings-body">
        <div class="status-row">
          <span class="field-label">当前后端</span>
          <n-tag size="small" :type="store.status?.active_error ? 'error' : 'success'">
            {{ backendLabel(store.status?.backend) }}
          </n-tag>
        </div>

        <n-alert v-if="store.status?.active_error" type="error" :show-icon="false">
          {{ store.status.active_error }}
        </n-alert>

        <div class="field-group">
          <span class="field-label">选择后端</span>
          <n-radio-group v-model:value="draftBackend" size="small">
            <n-radio-button value="sqlite">SQLite</n-radio-button>
            <n-radio-button value="yaml">YAML</n-radio-button>
          </n-radio-group>
        </div>

        <div v-if="draftBackend === 'yaml'" class="field-group">
          <span class="field-label">YAML 路径</span>
          <n-input
            v-model:value="yamlPath"
            placeholder="绝对路径，例如 D:\\w-ssh\\sessions.yml"
            clearable
          />
        </div>

        <div class="path-block">
          <span class="field-label">SQLite 路径</span>
          <code>{{ store.status?.sqlite_path || '—' }}</code>
        </div>

        <n-alert v-if="isChangingBackend" type="warning" :show-icon="false">
          直接应用只切换后端，不复制会话。复制并切换只接受空目标，成功后保留源数据。
        </n-alert>
      </div>
    </n-spin>

    <template #footer>
      <div class="modal-actions">
        <n-button @click="visible = false">关闭</n-button>
        <div class="primary-actions">
          <n-button
            v-if="isChangingBackend"
            :loading="store.copying"
            :disabled="busy || !selectionValid"
            @click="handleCopy"
          >
            <template #icon><n-icon :component="CopyOutline" /></template>
            复制并切换
          </n-button>
          <n-button
            type="primary"
            :loading="store.applying"
            :disabled="busy || !selectionValid"
            @click="handleApply"
          >
            <template #icon><n-icon :component="CheckmarkOutline" /></template>
            应用
          </n-button>
        </div>
      </div>
    </template>
  </n-modal>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import {
  NAlert,
  NButton,
  NIcon,
  NInput,
  NModal,
  NRadioButton,
  NRadioGroup,
  NSpin,
  NTag,
  useMessage,
} from 'naive-ui'
import { CheckmarkOutline, CopyOutline } from '@vicons/ionicons5'
import { useStorageStore } from '../stores/storage'
import type { StorageBackend, StorageSelection } from '../types'

const props = defineProps<{ modelValue: boolean }>()
const emit = defineEmits<{
  'update:modelValue': [boolean]
  changed: []
}>()

const store = useStorageStore()
const message = useMessage()
const visible = ref(props.modelValue)
const draftBackend = ref<StorageBackend>('sqlite')
const yamlPath = ref('')

const busy = computed(() => store.loading || store.applying || store.copying)
const isChangingBackend = computed(() => (
  store.status !== null && draftBackend.value !== store.status.backend
))
const selectionValid = computed(() => (
  store.status !== null
  && (draftBackend.value === 'sqlite' || yamlPath.value.trim().length > 0)
))

watch(() => props.modelValue, value => (visible.value = value))
watch(visible, value => emit('update:modelValue', value))
watch(
  () => store.status,
  status => {
    if (!status) return
    draftBackend.value = status.backend
    yamlPath.value = status.yaml_path || yamlPath.value
  },
  { immediate: true },
)

watch(
  () => props.modelValue,
  async open => {
    if (!open) return
    try {
      await store.fetchStatus()
      resetDraft()
    } catch (error) {
      message.error(`无法读取存储状态: ${error}`)
    }
  },
)

function backendLabel(backend?: StorageBackend) {
  return backend === 'yaml' ? 'YAML' : 'SQLite'
}

function selection(): StorageSelection {
  return {
    backend: draftBackend.value,
    yaml_path: yamlPath.value.trim() || store.status?.yaml_path || undefined,
  }
}

function resetDraft() {
  draftBackend.value = store.status?.backend ?? 'sqlite'
  yamlPath.value = store.status?.yaml_path ?? ''
}

async function handleApply() {
  try {
    await store.applySelection(selection())
    message.success('存储设置已应用')
    emit('changed')
  } catch (error) {
    message.error(`应用失败: ${error}`)
  }
}

async function handleCopy() {
  try {
    const result = await store.copyAndSwitch(selection())
    message.success(`已复制并校验 ${result.copied} 个会话`)
    emit('changed')
  } catch (error) {
    message.error(`复制未完成: ${error}`)
  }
}
</script>

<style scoped>
.storage-modal {
  width: min(620px, calc(100vw - 32px));
}

.settings-body {
  display: flex;
  flex-direction: column;
  gap: 18px;
  min-height: 240px;
}

.status-row,
.field-group {
  display: flex;
  align-items: center;
  gap: 14px;
}

.field-group .n-input {
  flex: 1;
  min-width: 0;
}

.field-label {
  width: 84px;
  flex-shrink: 0;
  color: #8899aa;
  font-size: 12px;
}

.path-block {
  display: grid;
  grid-template-columns: 84px minmax(0, 1fr);
  gap: 14px;
  align-items: start;
}

.path-block code {
  color: #c9d5e0;
  font-size: 12px;
  overflow-wrap: anywhere;
  white-space: normal;
}

.modal-actions,
.primary-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.modal-actions {
  justify-content: space-between;
}

@media (max-width: 520px) {
  .status-row,
  .field-group {
    align-items: stretch;
    flex-direction: column;
    gap: 8px;
  }

  .field-label {
    width: auto;
  }

  .path-block {
    grid-template-columns: 1fr;
    gap: 8px;
  }

  .modal-actions {
    align-items: stretch;
    flex-direction: column-reverse;
  }

  .primary-actions {
    align-items: stretch;
    flex-direction: column;
  }
}
</style>
