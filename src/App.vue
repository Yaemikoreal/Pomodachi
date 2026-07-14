<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import PetWindow from './components/PetWindow.vue';
import ChatBubble from './components/ChatBubble.vue';
import SettingsPanel from './components/SettingsPanel.vue';
import TaskPanel from './components/TaskPanel.vue';
import { sendMessage } from './llm/client';
import { createTask, type Task } from './task/client';
import { getSettings, type Settings, DEFAULT_SETTINGS } from './settings/client';
import { isTauri } from './utils/tauri';

const petRef = ref<InstanceType<typeof PetWindow> | null>(null);
const chatVisible = ref(false);
const settingsVisible = ref(false);
const taskVisible = ref(false);
const messages = ref<{ role: 'user' | 'assistant'; content: string }[]>([]);
const isThinking = ref(false);
const reminderMsg = ref('');
const settings = ref<Settings>({ ...DEFAULT_SETTINGS });

onMounted(async () => {
  settings.value = await getSettings();
  applyWindowSettings(settings.value);

  if (!isTauri) return;
  listen<Task>('reminder:trigger', (event) => {
    const task = event.payload;
    reminderMsg.value = task.title;
    petRef.value?.playEmotion(task.emotion_hint || '认真');
    chatVisible.value = true;
  });

  listen('open-settings', () => {
    settingsVisible.value = true;
  });
});

async function applyWindowSettings(s: Settings) {
  if (!isTauri) return;
  try {
    await invoke('set_always_on_top', { alwaysOnTop: s.always_on_top });
    await invoke('set_auto_start', { autoStart: s.auto_start });
  } catch (e) {
    console.error('Failed to apply window settings:', e);
  }
}

async function reloadSettings() {
  settings.value = await getSettings();
  await applyWindowSettings(settings.value);
}

function toggleChat() {
  chatVisible.value = !chatVisible.value;
}

async function handleSend(text: string) {
  messages.value.push({ role: 'user', content: text });
  isThinking.value = true;
  try {
    const resp = await sendMessage(text);
    messages.value.push({ role: 'assistant', content: resp.content });
    if (resp.emotion_tag) {
      petRef.value?.playEmotion(resp.emotion_tag);
    }
    if (resp.intent?.intent === 'create_task') {
      await createTask({
        title: resp.intent.title || text,
        due_at: resp.intent.due_at ?? undefined,
        emotion_hint: resp.intent.emotion_hint,
      });
      messages.value.push({
        role: 'assistant',
        content: `已经记下啦：${resp.intent.title || text}`,
      });
    }
  } catch (e: any) {
    messages.value.push({ role: 'assistant', content: '呜，出错了：' + e });
  } finally {
    isThinking.value = false;
  }
}

function dismissReminder() {
  reminderMsg.value = '';
}
</script>

<template>
  <div class="app-root" data-tauri-drag-region>
    <PetWindow
      ref="petRef"
      :scale-factor="settings.scale_factor"
      :click-through="settings.click_through"
      :render-fix="settings.render_fix"
      :avatar-pack-path="settings.avatar_pack_path"
      @dblclick="toggleChat"
    />

    <div v-if="reminderMsg" class="reminder-bubble" @click.stop>
      <div>该做啦：{{ reminderMsg }}</div>
      <button @click="dismissReminder">知道了</button>
    </div>

    <ChatBubble
      :visible="chatVisible"
      :loading="isThinking"
      @close="chatVisible = false"
      @send="handleSend"
    >
      <div class="messages">
        <div
          v-for="(msg, idx) in messages"
          :key="idx"
          :class="['msg', msg.role]"
        >
          {{ msg.content }}
        </div>
        <div v-if="isThinking" class="msg assistant thinking">流萤思考中...</div>
      </div>
    </ChatBubble>

    <SettingsPanel v-model:visible="settingsVisible" @saved="reloadSettings" />
    <TaskPanel v-model:visible="taskVisible" />

    <div class="toolbar">
      <button @click.stop="chatVisible = !chatVisible">聊天</button>
      <button @click.stop="taskVisible = !taskVisible">任务</button>
      <button @click.stop="settingsVisible = !settingsVisible">设置</button>
    </div>
  </div>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body, #app {
  width: 100vw;
  height: 100vh;
  overflow: hidden;
  background: transparent;
}

.app-root {
  width: 100vw;
  height: 100vh;
  position: relative;
  background: transparent;
  -webkit-app-region: drag;
}

input, button, .chat-bubble, .settings-panel, .task-panel, .reminder-bubble, .toolbar {
  -webkit-app-region: no-drag;
}

.toolbar {
  position: fixed;
  top: 8px;
  left: 8px;
  display: flex;
  gap: 6px;
  z-index: 70;
}

.toolbar button {
  padding: 4px 8px;
  font-size: 12px;
  cursor: pointer;
  border-radius: 6px;
  border: 1px solid rgba(255, 255, 255, 0.4);
  background: rgba(255, 255, 255, 0.8);
}

.messages {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.msg {
  padding: 8px 10px;
  border-radius: 10px;
  max-width: 90%;
  word-break: break-word;
  font-size: 13px;
}

.msg.user {
  align-self: flex-end;
  background: #ffe4e9;
}

.msg.assistant {
  align-self: flex-start;
  background: #f0f0f0;
}

.msg.thinking {
  color: #888;
  font-style: italic;
}

.reminder-bubble {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  background: rgba(255, 243, 224, 0.98);
  border-radius: 16px;
  padding: 16px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.2);
  z-index: 80;
  text-align: center;
  font-size: 14px;
  color: #333;
}

.reminder-bubble button {
  margin-top: 10px;
  padding: 6px 14px;
  border: none;
  border-radius: 8px;
  background: #ffab91;
  color: white;
  cursor: pointer;
}
</style>
