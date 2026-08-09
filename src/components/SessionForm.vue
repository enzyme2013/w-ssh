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
      <n-form-item label="主机图标">
        <IconSelect v-model="form.icon" />
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
        <n-select
          v-model:value="form.group_name"
          :options="groupOptions"
          :render-label="renderGroupLabel"
          filterable
          tag
          clearable
          placeholder="选择或输入新分组"
        />
      </n-form-item>
      <n-form-item label="认证方式">
        <n-radio-group v-model:value="authType">
          <n-radio value="password">密码</n-radio>
          <n-radio value="key">私钥</n-radio>
        </n-radio-group>
      </n-form-item>
      <n-form-item v-if="authType === 'password'" label="密码">
        <n-input
          v-model:value="form.secret"
          type="password"
          show-password-on="click"
          :placeholder="isEdit ? '留空则不修改已保存凭据' : '可留空，连接时再输入'"
        />
      </n-form-item>
      <template v-else>
        <n-form-item label="私钥路径" path="private_key">
          <n-select
            v-model:value="form.private_key"
            :options="keyOptions"
            filterable
            tag
            placeholder="选择或手动输入私钥路径"
            :loading="loadingKeys"
          />
        </n-form-item>
        <n-form-item label="密钥口令">
          <n-input
            v-model:value="form.secret"
            type="password"
            show-password-on="click"
            placeholder="Passphrase 可选，默认留空"
          />
        </n-form-item>
      </template>
      <n-form-item v-if="form.secret" label="凭据保存">
        <n-checkbox v-model:checked="saveCredential">
          保存到系统凭据库
        </n-checkbox>
      </n-form-item>
      <n-alert type="info" :show-icon="false" class="credential-alert">
        会话文件只保存连接资料；密码与 passphrase 不写入 SQLite 或 YAML。
      </n-alert>
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
import { h, ref, watch, computed } from 'vue'
import {
  NModal, NForm, NFormItem, NInput, NInputNumber, NSelect,
  NButton, NSpace, NRadioGroup, NRadio, NAlert, NCheckbox, NIcon,
  useMessage, type FormInst, type FormRules, type SelectOption,
} from 'naive-ui'
import { invoke } from '@tauri-apps/api/core'
import { useSessionsStore } from '../stores/sessions'
import type { Session } from '../types'
import IconSelect from './IconSelect.vue'
import { resolveSessionIcon } from '../utils/sessionIcons'

const props = defineProps<{
  modelValue: boolean
  session?: Session
  defaultGroup?: string
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
const saveCredential = ref(true)

// 系统私钥检测
const availableKeys = ref<string[]>([])
const loadingKeys = ref(false)
const fallbackKeyPaths = ['~/.ssh/id_ed25519', '~/.ssh/id_rsa']
const keyOptions = computed(() => {
  const keys = availableKeys.value.length > 0 ? availableKeys.value : fallbackKeyPaths
  return keys.map(key => ({
    label: `~/.ssh/${key.replace(/\\/g, '/').split('/').pop()}`,
    value: key,
  }))
})

// 切换到私钥模式时自动扫描 ~/.ssh/
watch(authType, async (type, previous) => {
  if (type !== previous) form.value.secret = ''
  if (type === 'key' && availableKeys.value.length === 0) {
    loadingKeys.value = true
    try {
      availableKeys.value = await invoke<string[]>('get_ssh_key_paths')
      if (!form.value.private_key) {
        form.value.private_key = availableKeys.value[0] || fallbackKeyPaths[0]
      }
    } catch {
      availableKeys.value = []
      if (!form.value.private_key) form.value.private_key = fallbackKeyPaths[0]
    } finally {
      loadingKeys.value = false
    }
  }
})
const formRef = ref<FormInst | null>(null)
const message = useMessage()
const store = useSessionsStore()
const groupOptions = computed(() => store.groups
  .filter(group => group.editable)
  .map(group => ({ label: group.name, value: group.name, icon: group.icon })))

function renderGroupLabel(option: SelectOption) {
  return h('span', { class: 'group-option' }, [
    h(NIcon, {
      component: resolveSessionIcon(String(option.icon || 'folder'), 'folder'),
      size: 16,
    }),
    h('span', String(option.label)),
  ])
}

const defaultForm = () => ({
  name: '',
  host: '',
  port: 22,
  username: 'root',
  secret: '',
  private_key: '',
  icon: 'server',
  group_name: props.defaultGroup || null as string | null,
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
        secret: '',
        private_key: s.private_key || '',
        icon: s.icon || 'server',
        group_name: s.group_name || null,
      }
      authType.value = s.auth_method === 'private_key' ? 'key' : 'password'
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
  saveCredential.value = true
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
      private_key: authType.value === 'key' ? form.value.private_key || undefined : undefined,
      auth_method: authType.value === 'key' ? 'private_key' as const : 'password' as const,
      icon: form.value.icon,
      group_name: form.value.group_name?.trim() || undefined,
      group_icon: store.groupIconForName(form.value.group_name?.trim()),
    }

    let saved: Session
    if (isEdit.value && props.session) {
      saved = await store.updateSession({ id: props.session.id, ...payload })
    } else {
      saved = await store.createSession(payload)
    }

    if (form.value.secret && saveCredential.value) {
      try {
        saved = await store.setCredential(
          saved.id,
          authType.value === 'key' ? 'private_key_passphrase' : 'password',
          form.value.secret,
        )
      } catch (error) {
        message.warning(`会话已保存，但系统凭据库写入失败：${error}`)
      }
    }

    message.success(isEdit.value ? '会话资料已保存' : '会话已创建')
    emit('saved', saved)
    visible.value = false
  } catch (e) {
    message.error(String(e))
  } finally {
    saving.value = false
  }
}
</script>

<style scoped>
.credential-alert {
  margin: 0 0 18px 80px;
  width: calc(100% - 80px);
}


:deep(.group-option) {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}
</style>
