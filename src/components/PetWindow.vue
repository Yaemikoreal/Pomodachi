<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { isTauri } from '../utils/tauri';
import { SpineRenderer } from '../spine/SpineRenderer';
import { loadAvatarConfig, type HitArea, type RenderHints } from '../spine/avatar-config';

const props = defineProps<{
  scaleFactor?: number;
  clickThrough?: boolean;
  renderFix?: boolean;
  avatarPackPath?: string;
}>();

const basePath = computed(() => {
  const p = props.avatarPackPath?.trim();
  return p && p !== '' ? p.replace(/\/$/, '') : '/avatar';
});

const emit = defineEmits<{ (e: 'dblclick'): void }>();

const spineContainer = ref<HTMLDivElement | null>(null);
const status = ref('加载中...');
let renderer: SpineRenderer | null = null;
let hitAreas: HitArea[] = [];
let renderHints: RenderHints | undefined;

// 点击穿透状态
let isHit = false;
let ignoreEvents = true;
let checkTimer: number | null = null;
let mouseDownPos: { x: number; y: number } | null = null;
let isDragging = false;

async function setIgnoreCursorEvents(ignore: boolean) {
  if (!isTauri) return;
  if (ignore === ignoreEvents) return;
  ignoreEvents = ignore;
  try {
    await invoke('set_ignore_cursor_events', { ignore });
  } catch (e) {
    console.error('Failed to set ignore cursor events:', e);
  }
}

async function setWindowSize(factor: number) {
  if (!isTauri) return;
  const size = SpineRenderer.computeWindowSize(factor);
  try {
    await invoke('set_window_size', { width: size.width, height: size.height });
  } catch (e) {
    console.error('Failed to set window size:', e);
  }
}

function checkHit(clientX: number, clientY: number) {
  if (!props.clickThrough) {
    if (!isHit) {
      isHit = true;
      setIgnoreCursorEvents(false);
    }
    return;
  }
  const hit = renderer?.hitTest(clientX, clientY) ?? false;
  if (hit !== isHit) {
    isHit = hit;
    setIgnoreCursorEvents(!hit);
  }
}

function scheduleHitCheck(clientX: number, clientY: number) {
  if (checkTimer) return;
  checkTimer = window.setTimeout(() => {
    checkTimer = null;
    checkHit(clientX, clientY);
  }, 100);
}

async function handleMouseMove(event: MouseEvent) {
  if (isDragging) return;

  // 如果正在按下并移动，触发系统拖拽
  if (mouseDownPos) {
    const dx = event.clientX - mouseDownPos.x;
    const dy = event.clientY - mouseDownPos.y;
    if (Math.sqrt(dx * dx + dy * dy) > 5) {
      isDragging = true;
      mouseDownPos = null;
      // 拖拽期间窗口必须接收鼠标事件
      await setIgnoreCursorEvents(false);
      if (isTauri) {
        try {
          await invoke('start_dragging');
        } catch (e) {
          console.error('Failed to start dragging:', e);
        }
      }
      return;
    }
  }

  scheduleHitCheck(event.clientX, event.clientY);
}

function handleMouseDown(event: MouseEvent) {
  if (isDragging) return;
  mouseDownPos = { x: event.clientX, y: event.clientY };
}

async function handleMouseUp() {
  if (isDragging) {
    isDragging = false;
    mouseDownPos = null;
    // 拖拽结束后根据当前命中状态恢复点击穿透
    await setIgnoreCursorEvents(!isHit);
  } else {
    mouseDownPos = null;
  }
}

function handleClick(event: MouseEvent) {
  if (isDragging) return;
  if (!renderer?.player || hitAreas.length === 0) return;
  const canvas = spineContainer.value?.querySelector('canvas');
  if (!canvas) return;
  const rect = canvas.getBoundingClientRect();
  const y = event.clientY - rect.top;
  const ratioY = y / rect.height;

  let target: HitArea | undefined;
  if (ratioY < 0.25) {
    target = hitAreas.find((h) => h.name.includes('头顶'));
  } else if (ratioY < 0.5) {
    target = hitAreas.find((h) => h.name.includes('脸') || h.name.includes('胸口'));
  } else if (ratioY < 0.75) {
    target = hitAreas.find((h) => h.name.includes('手'));
  } else {
    target = hitAreas.find((h) => h.name.includes('腿'));
  }

  if (target) {
    const motionMap: Record<string, string> = {
      头顶: 'Face_CloseEyesHappy',
      脸: 'Face_Happy',
      胸口: 'Face_Angry',
      右手: 'Hand_Akimbo',
      左手: 'Hand_Clap',
      左腿: 'Move_Sit_Idle',
      右腿: 'Move_Sit_Idle',
    };
    const anim = motionMap[target.name] || 'Face_Happy';
    renderer.play(anim, false);
    setTimeout(() => renderer?.player?.setAnimation('Move_Sit_Idle', true), 2000);
  }
}

onMounted(() => {
  if (!spineContainer.value) return;

  renderer = new SpineRenderer({
    containerId: 'spine-canvas',
    skelUrl: `${basePath.value}/skeleton.skel`,
    atlasUrl: `${basePath.value}/skeleton.atlas`,
    renderHints,
    renderFix: props.renderFix,
    onLoaded: async () => {
      status.value = '';
      const config = await loadAvatarConfig(`${basePath.value}/model0.json`);
      hitAreas = config?.hitAreas ?? [];
      renderHints = config?.renderHints;
      if (props.renderFix && renderer) {
        renderer.applyRenderFix(true, renderHints);
      }
      // 初始应用缩放
      const factor = props.scaleFactor ?? 1.0;
      renderer?.setScale(factor);
      setWindowSize(factor);
      // 根据设置初始化点击穿透状态
      setIgnoreCursorEvents(!props.clickThrough);
    },
    onError: (msg: string) => {
      status.value = '模型加载失败：' + msg;
    },
  });

  window.addEventListener('mousemove', handleMouseMove);
  window.addEventListener('mousedown', handleMouseDown);
  window.addEventListener('mouseup', handleMouseUp);
});

onUnmounted(() => {
  renderer?.destroy();
  renderer = null;
  window.removeEventListener('mousemove', handleMouseMove);
  window.removeEventListener('mousedown', handleMouseDown);
  window.removeEventListener('mouseup', handleMouseUp);
  if (checkTimer) window.clearTimeout(checkTimer);
});

watch(() => props.scaleFactor, (factor) => {
  const f = factor ?? 1.0;
  renderer?.setScale(f);
  setWindowSize(f);
});

watch(() => props.renderFix, (enabled) => {
  renderer?.applyRenderFix(enabled ?? false, renderHints);
});

watch(() => props.clickThrough, (enabled) => {
  if (!enabled) {
    isHit = true;
    setIgnoreCursorEvents(false);
  } else {
    isHit = false;
    setIgnoreCursorEvents(true);
  }
});

defineExpose({
  playEmotion: (tag: string) => renderer?.playEmotion(tag),
});
</script>

<template>
  <div class="pet-window" @dblclick="emit('dblclick')">
    <div
      id="spine-canvas"
      ref="spineContainer"
      class="spine-canvas"
      @click="handleClick"
    ></div>
    <div v-if="status" class="status">{{ status }}</div>
  </div>
</template>

<style scoped>
.pet-window {
  width: 100vw;
  height: 100vh;
  position: relative;
  overflow: hidden;
  background: transparent;
}

.spine-canvas {
  width: 100%;
  height: 100%;
}

.status {
  position: absolute;
  bottom: 8px;
  left: 50%;
  transform: translateX(-50%);
  color: #fff;
  font-size: 12px;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.6);
  pointer-events: none;
}
</style>
