import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useTasks } from '../hooks/useTasks';

const priorityColors: Record<number, string> = {
  0: '#aaa',
  1: '#f59e0b',
  2: '#ef4444',
};

const priorityLabels: Record<number, string> = {
  0: '普通',
  1: '重要',
  2: '紧急',
};

export function TaskPanel() {
  const { tasks, isLoading, error, addTask, completeTask, deleteTask } = useTasks();
  const [newTitle, setNewTitle] = useState('');
  const [newPriority, setNewPriority] = useState(0);
  const [showAdd, setShowAdd] = useState(false);

  const handleAdd = async () => {
    const title = newTitle.trim();
    if (!title) return;
    await addTask(title, newPriority);
    setNewTitle('');
    setNewPriority(0);
    setShowAdd(false);
  };

  const incompleteTasks = tasks.filter((t) => !t.completed);
  const completedTasks = tasks.filter((t) => t.completed);

  return (
    <div className="flex flex-col h-full">
      {/* 头部操作栏 */}
      <div className="p-3 border-b border-gray-100">
        <button
          onClick={() => setShowAdd(!showAdd)}
          className="w-full py-2 bg-blue-500 text-white rounded-lg text-sm font-medium hover:bg-blue-600 transition-colors"
        >
          + 添加任务
        </button>
      </div>

      {/* 添加表单 */}
      <AnimatePresence>
        {showAdd && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: 'auto', opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            className="overflow-hidden border-b border-gray-100"
          >
            <div className="p-3 space-y-2">
              <input
                type="text"
                value={newTitle}
                onChange={(e) => setNewTitle(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter' && !e.shiftKey) {
                    e.preventDefault();
                    handleAdd();
                  }
                  if (e.key === 'Escape') {
                    setShowAdd(false);
                  }
                }}
                placeholder="输入任务内容..."
                autoFocus
                className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-300"
              />
              <div className="flex items-center justify-between">
                <div className="flex gap-1">
                  {([0, 1, 2] as const).map((p) => (
                    <button
                      key={p}
                      onClick={() => setNewPriority(p)}
                      className={`px-2 py-0.5 text-xs rounded-full border transition-colors ${
                        newPriority === p
                          ? 'border-blue-500 bg-blue-50 text-blue-600'
                          : 'border-gray-300 text-gray-500 hover:border-gray-400'
                      }`}
                    >
                      {priorityLabels[p]}
                    </button>
                  ))}
                </div>
                <button
                  onClick={handleAdd}
                  disabled={!newTitle.trim()}
                  className="px-3 py-1 text-xs bg-blue-500 text-white rounded-lg disabled:opacity-40"
                >
                  确定
                </button>
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* 任务列表 */}
      <div className="flex-1 overflow-y-auto scrollbar-thin">
        {isLoading && (
          <div className="p-4 space-y-3">
            {[1, 2, 3].map((i) => (
              <div key={i} className="h-10 bg-gray-100 rounded-lg animate-pulse" />
            ))}
          </div>
        )}

        {error && (
          <div className="p-4 text-center">
            <p className="text-red-500 text-sm mb-2">加载失败</p>
            <button
              onClick={() => window.location.reload()}
              className="text-xs text-blue-500 underline"
            >
              重试
            </button>
          </div>
        )}

        {!isLoading && !error && tasks.length === 0 && (
          <div className="p-8 text-center">
            <div className="text-3xl mb-2">[]</div>
            <p className="text-gray-400 text-sm">还没有任务~</p>
            <p className="text-gray-300 text-xs mt-1">点击上方按钮添加一个吧</p>
          </div>
        )}

        {!isLoading && !error && incompleteTasks.length > 0 && (
          <div className="py-1">
            {incompleteTasks.map((task) => (
              <TaskItem
                key={task.id}
                title={task.title}
                priority={task.priority}
                completed={false}
                onToggle={() => completeTask(task.id, true)}
                onDelete={() => deleteTask(task.id)}
              />
            ))}
          </div>
        )}

        {completedTasks.length > 0 && (
          <>
            <div className="px-4 py-2 text-xs text-gray-400 border-t border-gray-100">
              已完成 ({completedTasks.length})
            </div>
            <div className="py-1">
              {completedTasks.map((task) => (
                <TaskItem
                  key={task.id}
                  title={task.title}
                  priority={task.priority}
                  completed={true}
                  onToggle={() => completeTask(task.id, false)}
                  onDelete={() => deleteTask(task.id)}
                />
              ))}
            </div>
          </>
        )}
      </div>
    </div>
  );
}

/** 单个任务项 */
function TaskItem({
  title,
  priority,
  completed,
  onToggle,
  onDelete,
}: {
  title: string;
  priority: number;
  completed: boolean;
  onToggle: () => void;
  onDelete: () => void;
}) {
  const [showDel, setShowDel] = useState(false);

  return (
    <div
      className="flex items-center gap-2 px-4 py-2 hover:bg-gray-50 group"
      onMouseEnter={() => setShowDel(true)}
      onMouseLeave={() => setShowDel(false)}
    >
      {/* 复选框 */}
      <button
        onClick={onToggle}
        className={`w-4 h-4 rounded-full border-2 flex-shrink-0 transition-colors ${
          completed
            ? 'bg-green-500 border-green-500'
            : 'border-gray-300 hover:border-blue-400'
        }`}
      >
        {completed && (
          <svg viewBox="0 0 16 16" className="w-full h-full text-white" fill="currentColor">
            <path d="M13.78 4.22a.75.75 0 010 1.06l-7.25 7.25a.75.75 0 01-1.06 0L2.22 9.28a.75.75 0 011.06-1.06L6 10.94l6.72-6.72a.75.75 0 011.06 0z" />
          </svg>
        )}
      </button>

      {/* 优先级标记 */}
      <span
        style={{ color: priorityColors[priority] || '#aaa' }}
        className="text-xs flex-shrink-0"
        title={priorityLabels[priority]}
      >
        ●
      </span>

      {/* 标题 */}
      <span
        className={`flex-1 text-sm truncate ${
          completed ? 'line-through text-gray-300' : 'text-gray-700'
        }`}
      >
        {title}
      </span>

      {/* 删除按钮 */}
      <AnimatePresence>
        {showDel && (
          <motion.button
            initial={{ opacity: 0, scale: 0.8 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0, scale: 0.8 }}
            onClick={onDelete}
            className="text-gray-300 hover:text-red-500 text-xs flex-shrink-0"
          >
            ✕
          </motion.button>
        )}
      </AnimatePresence>
    </div>
  );
}
