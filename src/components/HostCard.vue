<template>
  <div
    class="host-card"
    @dblclick="emit('connect', session)"
    @contextmenu.prevent="showContextMenu"
  >
    <div class="card-icon">
      <div class="icon-bubble">
        <n-icon :component="DesktopOutline" :size="22" color="#6b9cf8" />
      </div>
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
  gap: 12px;
  padding: 20px 14px 16px;
  background: #161b27;
  border: 1px solid #21283a;
  border-radius: 10px;
  cursor: pointer;
  user-select: none;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.4);
  transition: background 0.15s, border-color 0.2s, transform 0.15s, box-shadow 0.2s;
}

.host-card:hover {
  background: #1c2333;
  border-color: rgba(107, 156, 248, 0.5);
  transform: translateY(-2px);
  box-shadow:
    0 4px 16px rgba(0, 0, 0, 0.5),
    0 0 0 1px rgba(107, 156, 248, 0.2),
    0 0 20px rgba(107, 156, 248, 0.08);
}

.card-icon {
  display: flex;
  align-items: center;
  justify-content: center;
}

.icon-bubble {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  background: rgba(107, 156, 248, 0.12);
  border: 1px solid rgba(107, 156, 248, 0.2);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.2s, border-color 0.2s;
}

.host-card:hover .icon-bubble {
  background: rgba(107, 156, 248, 0.20);
  border-color: rgba(107, 156, 248, 0.35);
}

.card-info {
  width: 100%;
  text-align: center;
  min-width: 0;
}

.card-name {
  font-size: 13px;
  font-weight: 600;
  color: #e2e8f0;
  letter-spacing: 0.01em;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.card-host {
  font-size: 11px;
  color: #4a5568;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-top: 3px;
}
</style>
