<template>
  <n-config-provider :theme="darkTheme" :theme-overrides="themeOverrides">
    <n-message-provider>
      <div class="app-layout">
        <!-- 顶部统一 Tab 栏 -->
        <div class="top-tab-bar">
          <!-- Vaults 固定 Tab -->
          <div
            class="tab-item"
            :class="{ active: activeTopTab === 'vaults' }"
            @click="activeTopTab = 'vaults'"
          >
            <n-icon :component="GridOutline" class="tab-icon" />
            <span class="tab-name">Vaults</span>
          </div>

          <!-- 终端动态 Tab -->
          <div
            v-for="tab in terminalsStore.tabs"
            :key="tab.id"
            class="tab-item"
            :class="{ active: activeTopTab === tab.id, disconnected: !tab.connected }"
            @click="activeTopTab = tab.id"
          >
            <n-icon
              :component="tab.connected ? TerminalOutline : AlertCircleOutline"
              class="tab-icon"
            />
            <span class="tab-name">{{ tab.session_name }}</span>
            <n-button
              quaternary
              circle
              size="tiny"
              class="tab-close"
              @click.stop="handleCloseTab(tab.id)"
            >
              <template #icon><n-icon :component="CloseOutline" /></template>
            </n-button>
          </div>

          <!-- 弹性空白：将 + 按钮推到右端 -->
          <div class="tab-spacer" />

          <!-- 新建按钮 -->
          <div class="tab-new-btn" @click="activeTopTab = 'vaults'">
            <n-icon :component="AddOutline" />
          </div>
        </div>

        <!-- 内容区 -->
        <div class="content-area">
          <VaultsView v-show="activeTopTab === 'vaults'" />
          <TerminalPanel
            v-for="tab in terminalsStore.tabs"
            :key="tab.id"
            :terminal-id="tab.id"
            :active="tab.id === activeTopTab"
            v-show="activeTopTab === tab.id"
            @closed="terminalsStore.markDisconnected(tab.id)"
          />
        </div>
      </div>
    </n-message-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { NConfigProvider, NMessageProvider, NButton, NIcon, darkTheme } from 'naive-ui'
import type { GlobalThemeOverrides } from 'naive-ui'
import {
  GridOutline,
  TerminalOutline,
  AlertCircleOutline,
  CloseOutline,
  AddOutline,
} from '@vicons/ionicons5'
import VaultsView from './components/VaultsView.vue'
import TerminalPanel from './components/TerminalPanel.vue'
import { useTerminalsStore } from './stores/terminals'

const terminalsStore = useTerminalsStore()
const activeTopTab = ref<string>('vaults')

// 同步：终端 store 打开新终端时，自动切换到该 Tab
watch(
  () => terminalsStore.activeTabId,
  (newId) => {
    activeTopTab.value = newId ?? 'vaults'
  },
)

async function handleCloseTab(terminalId: string) {
  await terminalsStore.closeTab(terminalId)
  if (terminalsStore.tabs.length === 0) {
    activeTopTab.value = 'vaults'
  }
}

const themeOverrides: GlobalThemeOverrides = {
  common: {
    primaryColor: '#6b9cf8',
    primaryColorHover: '#93b4fb',
    primaryColorPressed: '#4a7cf0',
    bodyColor: '#0d1117',
    cardColor: '#161b27',
    modalColor: '#161b27',
    popoverColor: '#161b27',
    borderColor: '#21283a',
    textColor1: '#e2e8f0',
    textColor2: '#c9d5e0',
    textColor3: '#8899aa',
  },
}
</script>

<style>
* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

html, body, #app {
  width: 100%;
  height: 100%;
  overflow: hidden;
  background: #0d1117;
}
</style>

<style scoped>
.app-layout {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100vh;
}

/* 顶部 Tab 栏 */
.top-tab-bar {
  display: flex;
  align-items: center;
  height: 40px;
  flex-shrink: 0;
  background: #090c11;
  border-bottom: 1px solid #21283a;
  overflow-x: auto;
}

.top-tab-bar::-webkit-scrollbar {
  height: 2px;
}

.tab-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 0 14px;
  height: 100%;
  cursor: pointer;
  white-space: nowrap;
  color: #4a5568;
  font-size: 12px;
  transition: background 0.15s, color 0.15s;
}

.tab-item:hover {
  background: rgba(255, 255, 255, 0.04);
  color: #c9d5e0;
}

.tab-item.active {
  background: #0d1117;
  color: #e2e8f0;
  border-bottom: 2px solid #6b9cf8;
}

.tab-item.disconnected .tab-icon {
  color: #f87171;
}

.tab-name {
  max-width: 120px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.tab-close {
  opacity: 0;
  margin-left: 2px;
}

.tab-item:hover .tab-close {
  opacity: 1;
}

.tab-spacer {
  flex: 1;
}

.tab-new-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 100%;
  cursor: pointer;
  color: #4a5568;
  font-size: 18px;
  flex-shrink: 0;
  transition: color 0.15s, background 0.15s;
}

.tab-new-btn:hover {
  color: #c9d5e0;
  background: rgba(255, 255, 255, 0.04);
}

/* 内容区 */
.content-area {
  flex: 1;
  position: relative;
  overflow: hidden;
}

.content-area > * {
  position: absolute;
  inset: 0;
}
</style>
