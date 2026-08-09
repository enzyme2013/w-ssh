import type { Component } from 'vue'
import {
  BriefcaseOutline,
  CloudOutline,
  CubeOutline,
  DesktopOutline,
  FolderOpenOutline,
  FolderOutline,
  GlobeOutline,
  HardwareChipOutline,
  LayersOutline,
  ServerOutline,
  TerminalOutline,
} from '@vicons/ionicons5'

export const sessionIconOptions = [
  { label: '服务器', value: 'server' },
  { label: '桌面设备', value: 'desktop' },
  { label: '终端', value: 'terminal' },
  { label: '云主机', value: 'cloud' },
  { label: '芯片设备', value: 'chip' },
  { label: '全球节点', value: 'globe' },
  { label: '文件夹', value: 'folder' },
  { label: '打开的文件夹', value: 'folder-open' },
  { label: '工作环境', value: 'briefcase' },
  { label: '分层环境', value: 'layers' },
  { label: '容器', value: 'cube' },
]

const iconRegistry: Record<string, Component> = {
  server: ServerOutline,
  desktop: DesktopOutline,
  terminal: TerminalOutline,
  cloud: CloudOutline,
  chip: HardwareChipOutline,
  globe: GlobeOutline,
  folder: FolderOutline,
  'folder-open': FolderOpenOutline,
  briefcase: BriefcaseOutline,
  layers: LayersOutline,
  cube: CubeOutline,
}

export function resolveSessionIcon(icon: string | undefined, fallback = 'server') {
  return iconRegistry[icon || ''] || iconRegistry[fallback] || ServerOutline
}
