<template>
  <n-modal
    v-model:show="visible"
    :title="isEdit ? '编辑会话' : '新建会话'"
    preset="dialog"
    style="width: 480px"
    :show-icon="false"
    @after-leave="resetForm"
  >
    <n-form
      ref="formRef"
      :model="form"
      :rules="rules"
      label-placement="left"
      label-width="80px"
      require-mark-placement="right-hanging"
    >
      <n-form-item label="会话名称" path="name">
        <n-input v-model:value="form.name" placeholder="例如：生产服务器" />
      </n-form-item>
      <n-form-item label="主机地址" path="host">
        <n-input v-model:value="form.host" placeholder="IP 或域名" />
      </n-form-item>
      <n-form-item label="端口" path="port">
        <n-input-number v-model:value="form.port" :min="1" :max="65535" style="width:100%" />
      </n-form-item>
      <n-form-item label="用户名" path="username">
        <n-input v-model:value="form.username" placeholder="例如：root" />
      </n-form-item>
      <n-form-item label="分组" path="group_name">
        <n-input v-model:value="form.group_name" placeholder="可选，用于分组管理" />
      </n-form-item>
      <n-form-item label="认证方式">
        <n-radio-group v-model:value="authType">
          <n-radio value="password">密码</n-radio>
          <n-radio value="key">私钥</n-radio>
        </n-radio-group>
      </n-form-item>
      <n-form-item v-if="authType === 'password'" label="密码" path="password">
        <n-input
          v-model:value="form.password"
          type="password"
          show-password-on="click"
          placeholder="SSH 密码"
        />
      </n-form-item>
      <n-form-item v-else label="私钥路径" path="private_key">
        <n-select
          v-model:value="form.private_key"
          :options="keyOptions"
          filterable
          tag
          placeholder="选择或手动输入私钥路径"
          :loading="loadingKeys"
        />
      </n-form-item>
    </n-form>

    <template #action>
      <n-space justify="end">
        <n-button @click="visible = false">取消</n-button>
        <n-button type="primary" :loading="saving" @click="handleSubmit">
          {{ isEdit ? '保存' : '创建' }}
        </n-button>
      </n-space>
    </template>
  </n-modal>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import {
  NModal, NForm, NFormItem, NInput, NInputNumber, NSelect,
  NButton, NSpace, NRadioGroup, NRadio,
  useMessage, type FormInst, type FormRules,
} from 'naive-ui'
import { invoke } from '@tauri-apps/api/core'
import { useSessionsStore } from '../stores/sessions'
import type { Session } from '../types'

const props = defineProps<{
  modelValue: boolean
  session?: Session
}>()

const emit = defineEmits<{
  'update:modelValue': [boolean]
  saved: [Session]
}>()

const visible = ref(props.modelValue)
watch(() => props.modelValue, v => (visible.value = v))
watch(visible, v => emit('update:modelValue', v))

const isEdit = ref(false)
const authType = ref<'password' | 'key'>('password')
const saving = ref(false)

// 系统私钥检测
const availableKeys = ref<string[]>([])
const loadingKeys = ref(false)
const keyOptions = computed(() =>
  availableKeys.value.map(k => ({ label: k.replace(/\\/g, '/'), value: k }))
)

// 切换到私钥模式时自动扫描 ~/.ssh/
watch(authType, async (type) => {
  if (type === 'key' && availableKeys.value.length === 0) {
    loadingKeys.value = true
    try {
      availableKeys.value = await invoke<string[]>('get_ssh_key_paths')
      // 未编辑状态下自动选第一个
      if (!form.value.private_key && availableKeys.value.length > 0) {
        form.value.private_key = availableKeys.value[0]
      }
    } finally {
      loadingKeys.value = false
    }
  }
})
const formRef = ref<FormInst | null>(null)
const message = useMessage()
const store = useSessionsStore()

const defaultForm = () => ({
  name: '',
  host: '',
  port: 22,
  username: 'root',
  password: '',
  private_key: '',
  group_name: '',
})

const form = ref(defaultForm())

const rules: FormRules = {
  name: [{ required: true, message: '请输入会话名称' }],
  host: [{ required: true, message: '请输入主机地址' }],
  username: [{ required: true, message: '请输入用户名' }],
}

watch(
  () => props.session,
  (s) => {
    if (s) {
      isEdit.value = true
      form.value = {
        name: s.name,
        host: s.host,
        port: s.port,
        username: s.username,
        password: s.password || '',
        private_key: s.private_key || '',
        group_name: s.group_name || '',
      }
      authType.value = s.private_key ? 'key' : 'password'
    } else {
      isEdit.value = false
      form.value = defaultForm()
      authType.value = 'password'
    }
  },
  { immediate: true },
)

function resetForm() {
  form.value = defaultForm()
  isEdit.value = false
  authType.value = 'password'
}

async function handleSubmit() {
  try {
    await formRef.value?.validate()
  } catch {
    return
  }

  saving.value = true
  try {
    const payload = {
      name: form.value.name,
      host: form.value.host,
      port: form.value.port,
      username: form.value.username,
      password: authType.value === 'password' ? form.value.password || undefined : undefined,
      private_key: authType.value === 'key' ? form.value.private_key || undefined : undefined,
      group_name: form.value.group_name || undefined,
    }

    let saved: Session
    if (isEdit.value && props.session) {
      saved = await store.updateSession({ id: props.session.id, ...payload })
    } else {
      saved = await store.createSession(payload)
    }

    message.success(isEdit.value ? '保存成功' : '创建成功')
    emit('saved', saved)
    visible.value = false
  } catch (e) {
    message.error(String(e))
  } finally {
    saving.value = false
  }
}
</script>
