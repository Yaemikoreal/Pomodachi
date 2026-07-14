<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { getSettings, saveSettings, DEFAULT_SETTINGS, type Settings } from '../settings/client';

const visible = defineModel<boolean>('visible', { default: false });
const emit = defineEmits<{ (e: 'saved'): void }>();

const settings = ref<Settings>({ ...DEFAULT_SETTINGS });
const saving = ref(false);
const message = ref('');

onMounted(async () => {
  try {
    settings.value = await getSettings();
  } catch (e) {
    console.error(e);
  }
});

async function save() {
  saving.value = true;
  message.value = '';
  try {
    await saveSettings(settings.value);
    message.value = '已保存';
    emit('saved');
    setTimeout(() => (visible.value = false), 600);
  } catch (e: any) {
    message.value = '保存失败：' + e;
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <div v-if="visible" class="settings-panel" @click.stop>
    <div class="header">
      <span>设置</span>
      <button class="close" @click="visible = false">×</button>
    </div>
    <div class="body">
      <section>
        <h4>LLM 伙伴</h4>
        <label>
          Provider
          <select v-model="settings.provider">
            <option value="openai">OpenAI</option>
            <option value="custom">自定义 OpenAI 兼容</option>
          </select>
        </label>
        <label>
          API Base
          <input v-model="settings.api_base" type="text" />
        </label>
        <label>
          Model
          <input v-model="settings.model" type="text" />
        </label>
        <label>
          API Key
          <input v-model="settings.api_key" type="password" placeholder="留空则删除" />
        </label>
        <label>
          系统提示词
          <textarea v-model="settings.system_prompt" rows="3" placeholder="留空使用默认人设"></textarea>
        </label>
      </section>

      <section>
        <h4>角色形象</h4>
        <label>
          资源包路径
          <input v-model="settings.avatar_pack_path" type="text" placeholder="例如 /avatar 或自定义目录" />
        </label>
        <label>
          整体缩放
          <input v-model.number="settings.scale_factor" type="range" min="0.5" max="1.5" step="0.1" />
          <span class="range-value">{{ settings.scale_factor.toFixed(1) }}x</span>
        </label>
        <label class="checkbox">
          <input v-model="settings.render_fix" type="checkbox" />
          边缘修复（尝试消除模型白线/透明边框）
        </label>
      </section>

      <section>
        <h4>窗口生命周期</h4>
        <label class="checkbox">
          <input v-model="settings.auto_start" type="checkbox" />
          开机自启
        </label>
        <label class="checkbox">
          <input v-model="settings.always_on_top" type="checkbox" />
          总是置顶
        </label>
        <label class="checkbox">
          <input v-model="settings.click_through" type="checkbox" />
          空白区域点击穿透
        </label>
        <label>
          显示/隐藏快捷键
          <input v-model="settings.shortcut_show_hide" type="text" placeholder="例如 Ctrl+Shift+P（暂未启用）" />
        </label>
      </section>

      <div v-if="message" class="message">{{ message }}</div>
    </div>
    <div class="footer">
      <button :disabled="saving" @click="save">保存</button>
    </div>
  </div>
</template>

<style scoped>
.settings-panel {
  position: fixed;
  top: 20px;
  left: 50%;
  transform: translateX(-50%);
  width: 300px;
  background: rgba(255, 255, 255, 0.96);
  border-radius: 16px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.2);
  z-index: 60;
  font-size: 13px;
  color: #333;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 14px;
  background: #e0f7fa;
  border-radius: 16px 16px 0 0;
  font-weight: 600;
}

.close {
  background: transparent;
  border: none;
  font-size: 20px;
  cursor: pointer;
}

.body {
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  max-height: 70vh;
  overflow-y: auto;
}

section h4 {
  margin: 0 0 8px;
  font-size: 13px;
  color: #555;
  border-bottom: 1px solid #eee;
  padding-bottom: 4px;
}

section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

label {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

label.checkbox {
  flex-direction: row;
  align-items: center;
  gap: 6px;
}

label.checkbox input {
  width: auto;
}

input,
select,
textarea {
  border: 1px solid #ddd;
  border-radius: 8px;
  padding: 6px 8px;
  font-family: inherit;
  font-size: 13px;
}

input[type='range'] {
  padding: 0;
}

.range-value {
  align-self: flex-end;
  font-size: 12px;
  color: #666;
}

.footer {
  padding: 10px 14px;
  display: flex;
  justify-content: flex-end;
  border-top: 1px solid #f0f0f0;
}

button {
  border: none;
  border-radius: 8px;
  padding: 6px 14px;
  background: #4fc3f7;
  color: white;
  cursor: pointer;
}

.message {
  color: #2e7d32;
  font-size: 12px;
}
</style>
