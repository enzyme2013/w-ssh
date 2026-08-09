<template>
  <div class="icon-picker" role="radiogroup" aria-label="选择图标">
    <button
      v-for="option in sessionIconOptions"
      :key="option.value"
      type="button"
      role="radio"
      class="icon-choice"
      :class="{ selected: modelValue === option.value }"
      :aria-label="option.label"
      :aria-checked="modelValue === option.value"
      @click="emit('update:modelValue', option.value)"
    >
      <n-icon :component="resolveSessionIcon(option.value)" :size="18" />
    </button>
  </div>
</template>

<script setup lang="ts">
import { NIcon } from 'naive-ui'
import { resolveSessionIcon, sessionIconOptions } from '../utils/sessionIcons'

defineProps<{
  modelValue?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [string]
}>()
</script>

<style scoped>
.icon-picker {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(34px, 1fr));
  gap: 6px;
  width: 100%;
}

.icon-choice {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 34px;
  min-width: 34px;
  padding: 0;
  color: #8899aa;
  background: #202636;
  border: 1px solid transparent;
  border-radius: 7px;
  cursor: pointer;
  transition: color 0.15s, background 0.15s, border-color 0.15s;
}

.icon-choice:hover {
  color: #c9d5e0;
  background: #283044;
}

.icon-choice.selected {
  color: #93b4fb;
  background: rgba(107, 156, 248, 0.14);
  border-color: rgba(107, 156, 248, 0.55);
}

.icon-choice:focus-visible {
  outline: 2px solid #93b4fb;
  outline-offset: 2px;
}
</style>
