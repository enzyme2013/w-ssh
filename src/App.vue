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
    primaryColor: '#89b4fa',
    primaryColorHover: '#b4befe',
    primaryColorPressed: '#74c7ec',
    bodyColor: '#1e1e2e',
    cardColor: '#313244',
    modalColor: '#313244',
    popoverColor: '#313244',
    borderColor: '#45475a',
    textColor1: '#cdd6f4',
    textColor2: '#bac2de',
    textColor3: '#a6adc8',
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
  background: #1e1e2e;
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
  height: 38px;
  flex-shrink: 0;
  background: #181825;
  border-bottom: 1px solid #313244;
  overflow-x: auto;
}

.top-tab-bar::-webkit-scrollbar {
  height: 2px;
}

.tab-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 0 12px;
  height: 100%;
  cursor: pointer;
  white-space: nowrap;
  color: #6c7086;
  border-right: 1px solid #313244;
  font-size: 13px;
  transition: background 0.15s, color 0.15s;
}

.tab-item:hover {
  background: #313244;
  color: #cdd6f4;
}

.tab-item.active {
  background: #1e1e2e;
  color: #cdd6f4;
  border-bottom: 2px solid #89b4fa;
}

.tab-item.disconnected .tab-icon {
  color: #f38ba8;
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
  color: #6c7086;
  font-size: 18px;
  flex-shrink: 0;
  transition: color 0.15s, background 0.15s;
}

.tab-new-btn:hover {
  color: #cdd6f4;
  background: #313244;
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
