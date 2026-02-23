<template>
  <div
    class="host-card"
    @dblclick="emit('connect', session)"
    @contextmenu.prevent="showContextMenu"
  >
    <div class="card-icon">
      <n-icon :component="DesktopOutline" :size="28" color="#89b4fa" />
    </div>
    <div class="card-info">
      <div class="card-name">{{ session.name }}</div>
      <div class="card-host">{{ session.username }}@{{ session.host }}</div>
    </div>

    <!-- 右键菜单 -->
    <n-dropdown
      trigger="manual"
      :x="ctxX"
      :y="ctxY"
      :options="ctxOptions"
      :show="ctxVisible"
      @select="handleSelect"
      @clickoutside="ctxVisible = false"
    />
  </div>
</template>

<script setup lang="ts">
import { h, ref } from 'vue'
import { NIcon, NDropdown, type DropdownOption } from 'naive-ui'
import {
  DesktopOutline,
  TerminalOutline,
  CreateOutline,
  TrashOutline,
} from '@vicons/ionicons5'
import type { Session } from '../types'

const props = defineProps<{ session: Session }>()
const emit = defineEmits<{
  connect: [session: Session]
  edit: [session: Session]
  delete: [session: Session]
}>()

const ctxVisible = ref(false)
const ctxX = ref(0)
const ctxY = ref(0)

const ctxOptions: DropdownOption[] = [
  {
    label: '连接',
    key: 'connect',
    icon: () => h(NIcon, null, { default: () => h(TerminalOutline) }),
  },
  {
    label: '编辑',
    key: 'edit',
    icon: () => h(NIcon, null, { default: () => h(CreateOutline) }),
  },
  { type: 'divider', key: 'd' },
  {
    label: '删除',
    key: 'delete',
    icon: () => h(NIcon, null, { default: () => h(TrashOutline) }),
  },
]

function showContextMenu(e: MouseEvent) {
  ctxX.value = e.clientX
  ctxY.value = e.clientY
  ctxVisible.value = true
}

function handleSelect(key: string) {
  ctxVisible.value = false
  if (key === 'connect') emit('connect', props.session)
  else if (key === 'edit') emit('edit', props.session)
  else if (key === 'delete') emit('delete', props.session)
}
</script>

<style scoped>
.host-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  padding: 16px 12px;
  background: #313244;
  border: 1px solid #45475a;
  border-radius: 8px;
  cursor: pointer;
  user-select: none;
  transition: background 0.15s, border-color 0.15s, transform 0.15s;
}

.host-card:hover {
  background: #3c3d52;
  border-color: #89b4fa;
  transform: translateY(-1px);
}

.card-icon {
  display: flex;
  align-items: center;
  justify-content: center;
}

.card-info {
  width: 100%;
  text-align: center;
  min-width: 0;
}

.card-name {
  font-size: 14px;
  font-weight: 600;
  color: #cdd6f4;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.card-host {
  font-size: 12px;
  color: #6c7086;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-top: 2px;
}
</style>
