<script setup lang="ts">
import { onMounted, ref, computed } from 'vue';
import {
  listTasks,
  completeTask,
  deleteTask,
  postponeTask,
  createTask,
  updateTask,
  type Task,
} from '../task/client';

const visible = defineModel<boolean>('visible', { default: false });
const tasks = ref<Task[]>([]);
const filter = ref<'all' | 'pending' | 'done'>('pending');
const editingTask = ref<Task | null>(null);
const showForm = ref(false);
const form = ref({
  title: '',
  due_at: '',
  repeat_rule: 'none',
  emotion_hint: '认真',
});
const postponeMinutes = ref<Record<number, number>>({});

onMounted(load);

async function load() {
  tasks.value = await listTasks(filter.value === 'all' || filter.value === 'done');
}

async function applyFilter() {
  await load();
}

const filteredTasks = computed(() => {
  if (filter.value === 'all') return tasks.value;
  return tasks.value.filter((t) => (filter.value === 'done' ? t.is_done : !t.is_done));
});

function isOverdue(task: Task): boolean {
  if (task.is_done || !task.due_at) return false;
  return task.due_at < Date.now();
}

async function done(id: number) {
  await completeTask(id);
  await load();
}

async function remove(id: number) {
  await deleteTask(id);
  await load();
}

async function later(id: number) {
  const minutes = postponeMinutes.value[id] ?? 5;
  await postponeTask(id, minutes);
  postponeMinutes.value[id] = 5;
  await load();
}

function startCreate() {
  editingTask.value = null;
  form.value = {
    title: '',
    due_at: '',
    repeat_rule: 'none',
    emotion_hint: '认真',
  };
  showForm.value = true;
}

function startEdit(task: Task) {
  editingTask.value = task;
  form.value = {
    title: task.title,
    due_at: task.due_at ? new Date(task.due_at).toISOString().slice(0, 16) : '',
    repeat_rule: task.repeat_rule || 'none',
    emotion_hint: task.emotion_hint || '认真',
  };
  showForm.value = true;
}

function cancelForm() {
  showForm.value = false;
  editingTask.value = null;
}

async function submitForm() {
  const dueAt = form.value.due_at ? new Date(form.value.due_at).getTime() : undefined;
  if (editingTask.value) {
    await updateTask({
      id: editingTask.value.id,
      title: form.value.title,
      due_at: dueAt,
      repeat_rule: form.value.repeat_rule,
      emotion_hint: form.value.emotion_hint,
    });
  } else {
    await createTask({
      title: form.value.title,
      due_at: dueAt,
      repeat_rule: form.value.repeat_rule,
      emotion_hint: form.value.emotion_hint,
    });
  }
  showForm.value = false;
  editingTask.value = null;
  await load();
}

function formatTime(ts?: number) {
  if (!ts) return '无时间';
  return new Date(ts).toLocaleString('zh-CN');
}

defineExpose({ load });
</script>

<template>
  <div v-if="visible" class="task-panel" @click.stop>
    <div class="header">
      <span>任务清单</span>
      <button class="close" @click="visible = false">×</button>
    </div>

    <div class="toolbar">
      <select v-model="filter" @change="applyFilter">
        <option value="pending">未完成</option>
        <option value="done">已完成</option>
        <option value="all">全部</option>
      </select>
      <button class="add" @click="startCreate">+ 新建</button>
    </div>

    <div class="body">
      <div v-if="showForm" class="task-form">
        <input v-model="form.title" type="text" placeholder="任务标题" />
        <input v-model="form.due_at" type="datetime-local" />
        <input v-model="form.emotion_hint" type="text" placeholder="情绪提示，例如 认真" />
        <select v-model="form.repeat_rule">
          <option value="none">不重复</option>
          <option value="daily">每天</option>
          <option value="weekly">每周</option>
        </select>
        <div class="form-actions">
          <button @click="cancelForm">取消</button>
          <button class="primary" @click="submitForm">{{ editingTask ? '更新' : '添加' }}</button>
        </div>
      </div>

      <div v-if="filteredTasks.length === 0" class="empty">暂无任务</div>
      <div
        v-for="task in filteredTasks"
        :key="task.id"
        :class="['task', { overdue: isOverdue(task), done: task.is_done }]"
      >
        <div class="info">
          <div class="title">{{ task.title }}</div>
          <div class="meta">
            <span class="time">{{ formatTime(task.due_at) }}</span>
            <span v-if="task.repeat_rule && task.repeat_rule !== 'none'" class="repeat">
              {{ task.repeat_rule }}
            </span>
            <span v-if="task.emotion_hint" class="emotion">{{ task.emotion_hint }}</span>
          </div>
        </div>
        <div class="actions">
          <template v-if="!task.is_done">
            <button @click="done(task.id)">完成</button>
            <button @click="startEdit(task)">编辑</button>
            <div class="postpone">
              <input
                v-model.number="postponeMinutes[task.id]"
                type="number"
                min="1"
                placeholder="分"
              />
              <button @click="later(task.id)">延后</button>
            </div>
          </template>
          <button class="danger" @click="remove(task.id)">删除</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.task-panel {
  position: fixed;
  top: 20px;
  right: 20px;
  width: 320px;
  max-height: 480px;
  background: rgba(255, 255, 255, 0.96);
  border-radius: 16px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.2);
  z-index: 55;
  font-size: 13px;
  color: #333;
  display: flex;
  flex-direction: column;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 14px;
  background: #fff9c4;
  border-radius: 16px 16px 0 0;
  font-weight: 600;
}

.close {
  background: transparent;
  border: none;
  font-size: 20px;
  cursor: pointer;
}

.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 14px;
  border-bottom: 1px solid #f0f0f0;
}

.toolbar select {
  border: 1px solid #ddd;
  border-radius: 8px;
  padding: 4px 6px;
}

.add {
  border: none;
  border-radius: 8px;
  padding: 4px 10px;
  background: #a5d6a7;
  color: white;
  cursor: pointer;
}

.body {
  padding: 10px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.empty {
  color: #999;
  text-align: center;
  padding: 20px 0;
}

.task-form {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 8px;
  background: #f5f5f5;
  border-radius: 10px;
}

.task-form input,
.task-form select {
  border: 1px solid #ddd;
  border-radius: 6px;
  padding: 5px 8px;
  font-size: 12px;
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 6px;
}

.form-actions button {
  border: none;
  border-radius: 6px;
  padding: 5px 12px;
  font-size: 12px;
  cursor: pointer;
  background: #e0e0e0;
}

.form-actions button.primary {
  background: #4fc3f7;
  color: white;
}

.task {
  background: #f9f9f9;
  border-radius: 10px;
  padding: 8px;
}

.task.overdue {
  border-left: 4px solid #ef9a9a;
}

.task.done {
  opacity: 0.7;
}

.title {
  font-weight: 500;
}

.meta {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 4px;
  font-size: 11px;
}

.time {
  color: #666;
}

.repeat,
.emotion {
  padding: 1px 5px;
  border-radius: 4px;
  background: #e3f2fd;
  color: #1976d2;
}

.actions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 6px;
}

.actions button {
  border: none;
  border-radius: 6px;
  padding: 4px 10px;
  font-size: 11px;
  cursor: pointer;
  background: #e0e0e0;
}

.actions button:first-child {
  background: #a5d6a7;
}

.actions button.danger {
  background: #ef9a9a;
  color: white;
}

.postpone {
  display: flex;
  gap: 4px;
  align-items: center;
}

.postpone input {
  width: 50px;
  border: 1px solid #ddd;
  border-radius: 6px;
  padding: 4px;
  font-size: 11px;
}

.postpone button {
  background: #ffe082;
}
</style>
