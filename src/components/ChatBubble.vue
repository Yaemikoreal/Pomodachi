<script setup lang="ts">
import { ref } from 'vue';

defineProps<{ visible: boolean; loading?: boolean }>();
const emit = defineEmits<{ (e: 'send', text: string): void; (e: 'close'): void }>();

const input = ref('');

function send() {
  const text = input.value.trim();
  if (!text) return;
  emit('send', text);
  input.value = '';
}
</script>

<template>
  <div v-if="visible" class="chat-bubble" @click.stop>
    <div class="header">
      <span>和流萤聊天</span>
      <button class="close" @click="emit('close')">×</button>
    </div>
    <div class="body">
      <slot />
    </div>
    <div class="footer">
      <input
        v-model="input"
        type="text"
        placeholder="说点什么..."
        @keydown.enter="send"
      />
      <button @click="send">发送</button>
    </div>
  </div>
</template>

<style scoped>
.chat-bubble {
  position: fixed;
  bottom: 20px;
  left: 50%;
  transform: translateX(-50%);
  width: 320px;
  background: rgba(255, 255, 255, 0.95);
  border-radius: 16px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.2);
  overflow: hidden;
  font-family: system-ui, -apple-system, sans-serif;
  color: #333;
  z-index: 50;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 14px;
  background: #ffe4e9;
  font-weight: 600;
  font-size: 14px;
}

.close {
  background: transparent;
  border: none;
  font-size: 20px;
  cursor: pointer;
  line-height: 1;
}

.body {
  min-height: 80px;
  max-height: 200px;
  overflow-y: auto;
  padding: 12px;
  font-size: 13px;
}

.footer {
  display: flex;
  gap: 8px;
  padding: 10px 12px;
  border-top: 1px solid #f0f0f0;
}

.footer input {
  flex: 1;
  border: 1px solid #ddd;
  border-radius: 8px;
  padding: 6px 10px;
  font-size: 13px;
}

.footer button {
  border: none;
  border-radius: 8px;
  padding: 6px 12px;
  background: #ff8fa3;
  color: white;
  cursor: pointer;
  font-size: 13px;
}
</style>
