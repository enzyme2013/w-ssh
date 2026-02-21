<template>
  <n-config-provider :theme="darkTheme" :theme-overrides="themeOverrides">
    <n-message-provider>
      <div class="app-layout">
        <!-- 左侧会话侧边栏 -->
        <div class="sidebar-wrap">
          <SessionSidebar />
        </div>

        <!-- 右侧终端区域 -->
        <div class="main-area">
          <!-- 无连接时的欢迎页 -->
          <div v-if="terminalsStore.tabs.length === 0" class="welcome">
            <div class="welcome-icon">⌨</div>
            <div class="welcome-title">w-ssh</div>
            <div class="welcome-hint">双击左侧会话以建立连接</div>
          </div>

          <!-- 多标签终端 -->
          <template v-else>
            <!-- 标签栏 -->
            <div class="tab-bar">
              <div
                v-for="tab in terminalsStore.tabs"
                :key="tab.id"
                class="tab-item"
                :class="{ active: tab.id === terminalsStore.activeTabId, disconnected: !tab.connected }"
                @click="terminalsStore.activeTabId = tab.id"
              >
                <n-icon :component="tab.connected ? TerminalOutline : AlertCircleOutline" class="tab-icon" />
                <span class="tab-name">{{ tab.session_name }}</span>
                <n-button
                  quaternary
                  circle
                  size="tiny"
                  class="tab-close"
                  @click.stop="terminalsStore.closeTab(tab.id)"
                >
                  <template #icon><n-icon :component="CloseOutline" /></template>
                </n-button>
              </div>
            </div>

            <!-- 终端内容区 -->
            <div class="terminal-area">
              <TerminalPanel
                v-for="tab in terminalsStore.tabs"
                :key="tab.id"
                :terminal-id="tab.id"
                :active="tab.id === terminalsStore.activeTabId"
                :style="{ display: tab.id === terminalsStore.activeTabId ? 'block' : 'none' }"
                @closed="terminalsStore.markDisconnected(tab.id)"
              />
            </div>
          </template>
        </div>
      </div>
    </n-message-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
import { NConfigProvider, NMessageProvider, NButton, NIcon, darkTheme } from 'naive-ui'
import type { GlobalThemeOverrides } from 'naive-ui'
import { TerminalOutline, AlertCircleOutline, CloseOutline } from '@vicons/ionicons5'
import SessionSidebar from './components/SessionSidebar.vue'
import TerminalPanel from './components/TerminalPanel.vue'
import { useTerminalsStore } from './stores/terminals'

const terminalsStore = useTerminalsStore()

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
  width: 100%;
  height: 100vh;
}

.sidebar-wrap {
  width: 220px;
  flex-shrink: 0;
  border-right: 1px solid #313244;
  overflow: hidden;
}

.main-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: #1e1e2e;
}

.welcome {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: #6c7086;
  gap: 12px;
}

.welcome-icon {
  font-size: 48px;
  opacity: 0.3;
}

.welcome-title {
  font-size: 24px;
  font-weight: 600;
  color: #45475a;
}

.welcome-hint {
  font-size: 14px;
}

/* 标签栏 */
.tab-bar {
  display: flex;
  align-items: center;
  background: #181825;
  border-bottom: 1px solid #313244;
  overflow-x: auto;
  flex-shrink: 0;
  height: 36px;
}

.tab-bar::-webkit-scrollbar {
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
  min-width: 0;
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
  max-width: 100px;
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

/* 终端区 */
.terminal-area {
  flex: 1;
  overflow: hidden;
}

.terminal-area > * {
  width: 100%;
  height: 100%;
}
</style>
