<template>
  <n-modal v-model:show="visible" preset="card" title="主机信任" class="trust-modal" :bordered="false">
    <n-spin :show="loading">
      <n-alert v-if="error" type="error" :show-icon="false">{{ error }}</n-alert>
      <div v-else-if="entries.length === 0" class="empty">尚未信任任何主机</div>
      <div v-else class="trust-list">
        <div v-for="entry in entries" :key="`${entry.endpoint}-${entry.algorithm}`" class="trust-row">
          <div class="trust-main">
            <strong>{{ entry.endpoint }}</strong>
            <span>{{ entry.algorithm }}</span>
            <code>{{ entry.fingerprint }}</code>
          </div>
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button
                quaternary
                circle
                type="error"
                aria-label="删除主机信任"
                @click="requestDelete(entry.endpoint)"
              >
                <template #icon><n-icon :component="TrashOutline" /></template>
              </n-button>
            </template>
            删除主机信任
          </n-tooltip>
        </div>
      </div>
    </n-spin>

    <n-modal v-model:show="showDelete" preset="dialog" title="删除主机信任" type="warning">
      <span>删除 {{ deletingEndpoint }} 的已信任指纹？下次连接将重新确认。</span>
      <template #action>
        <n-space justify="end">
          <n-button @click="showDelete = false">取消</n-button>
          <n-button type="error" :loading="deleting" @click="confirmDelete">删除</n-button>
        </n-space>
      </template>
    </n-modal>
  </n-modal>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import {
  NAlert, NButton, NIcon, NModal, NSpace, NSpin, NTooltip, useMessage,
} from 'naive-ui'
import { TrashOutline } from '@vicons/ionicons5'
import type { HostTrustEntry } from '../types'

const props = defineProps<{ modelValue: boolean }>()
const emit = defineEmits<{ 'update:modelValue': [boolean] }>()
const visible = ref(props.modelValue)
const entries = ref<HostTrustEntry[]>([])
const loading = ref(false)
const error = ref('')
const showDelete = ref(false)
const deleting = ref(false)
const deletingEndpoint = ref('')
const message = useMessage()

watch(() => props.modelValue, value => (visible.value = value))
watch(visible, value => emit('update:modelValue', value))
watch(() => props.modelValue, open => {
  if (open) void loadEntries()
})

async function loadEntries() {
  loading.value = true
  error.value = ''
  try {
    entries.value = await invoke<HostTrustEntry[]>('get_host_trust_entries')
  } catch (reason) {
    error.value = String(reason)
  } finally {
    loading.value = false
  }
}

function requestDelete(endpoint: string) {
  deletingEndpoint.value = endpoint
  showDelete.value = true
}

async function confirmDelete() {
  deleting.value = true
  try {
    await invoke('delete_host_trust', { endpoint: deletingEndpoint.value })
    showDelete.value = false
    await loadEntries()
    message.success('主机信任已删除')
  } catch (reason) {
    message.error(`删除失败: ${reason}`)
  } finally {
    deleting.value = false
  }
}
</script>

<style scoped>
.trust-modal { width: min(680px, calc(100vw - 32px)); }
.empty { min-height: 180px; display: grid; place-items: center; color: #8899aa; }
.trust-list { display: flex; flex-direction: column; }
.trust-row { display: flex; align-items: center; gap: 12px; padding: 14px 0; border-bottom: 1px solid #21283a; }
.trust-main { display: grid; min-width: 0; flex: 1; gap: 4px; }
.trust-main strong { font-size: 13px; }
.trust-main span { color: #8899aa; font-size: 12px; }
.trust-main code { color: #c9d5e0; overflow-wrap: anywhere; font-size: 12px; }
</style>
