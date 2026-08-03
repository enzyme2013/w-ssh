import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type {
  StorageBackend,
  StorageCopyResult,
  StorageSelection,
  StorageStatus,
} from '../types'

export const useStorageStore = defineStore('storage', () => {
  const status = ref<StorageStatus | null>(null)
  const loading = ref(false)
  const applying = ref(false)
  const copying = ref(false)

  const backend = computed<StorageBackend>(() => status.value?.backend ?? 'sqlite')

  async function fetchStatus() {
    loading.value = true
    try {
      status.value = await invoke<StorageStatus>('get_storage_status')
      return status.value
    } finally {
      loading.value = false
    }
  }

  async function applySelection(selection: StorageSelection) {
    applying.value = true
    try {
      status.value = await invoke<StorageStatus>('set_storage_settings', { selection })
      return status.value
    } finally {
      applying.value = false
    }
  }

  async function copyAndSwitch(selection: StorageSelection) {
    copying.value = true
    try {
      const result = await invoke<StorageCopyResult>('copy_storage_and_switch', { selection })
      status.value = result.status
      return result
    } finally {
      copying.value = false
    }
  }

  return { status, backend, loading, applying, copying, fetchStatus, applySelection, copyAndSwitch }
})
