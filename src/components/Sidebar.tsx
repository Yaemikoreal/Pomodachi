import { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { ChatPanel } from './ChatPanel';
import { TaskPanel } from './TaskPanel';
import { useChat } from '../hooks/useChat';
import { useSettings, type AppSettings } from '../hooks/useSettings';
import { useTasks } from '../hooks/useTasks';

type SidebarTab = 'chat' | 'tasks' | 'settings';

interface SidebarProps {
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
}

export function Sidebar({ open, onOpenChange }: SidebarProps) {
  const [internalOpen, setInternalOpen] = useState(false);
  const [activeTab, setActiveTab] = useState<SidebarTab>('chat');

  const isOpen = open ?? internalOpen;
  const setIsOpen = (val: boolean) => {
    if (onOpenChange) {
      onOpenChange(val);
    } else {
      setInternalOpen(val);
    }
  };
  const { clearAiSession } = useChat();
  const { settings, isLoading, isSaving, loadSettings, saveSettings } = useSettings();

  // 本地编辑态（修改后手动保存）
  const [editSettings, setEditSettings] = useState<AppSettings | null>(null);
  const [claudeStatus, setClaudeStatus] = useState<'未知' | '已检测' | '未安装'>('未知');

  // 加载设置
  useEffect(() => {
    if (isOpen && activeTab === 'settings') {
      loadSettings();
      checkClaude();
    }
  }, [isOpen, activeTab, loadSettings]);

  // 编辑态随加载结果同步
  useEffect(() => {
    if (settings) {
      setEditSettings({ ...settings });
    }
  }, [settings]);

  const checkClaude = async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('clear_ai_session');
      setClaudeStatus('已检测');
    } catch {
      setClaudeStatus('未知');
    }
  };

  // 保存配置
  const handleSave = async () => {
    if (!editSettings) return;
    await saveSettings(editSettings);
  };

  // 编辑辅助
  const updateSetting = <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => {
    setEditSettings((prev) => (prev ? { ...prev, [key]: value } : prev));
  };

  // 任务未读计数
  const { pendingCount: pendingTasks } = useTasks();

  return (
    <>
      {/* 切换按钮 */}
      <motion.button
        className="fixed right-4 top-4 z-50 w-10 h-10 bg-white/90 backdrop-blur-sm rounded-full shadow-lg flex items-center justify-center text-xl hover:scale-110 transition-transform"
        whileHover={{ scale: 1.1 }}
        whileTap={{ scale: 0.9 }}
        onClick={() => setIsOpen(!isOpen)}
      >
        {isOpen ? '✕' : '💬'}
      </motion.button>

      {/* 侧边栏 */}
      <AnimatePresence>
        {isOpen && (
          <motion.div
            initial={{ x: 320, opacity: 0 }}
            animate={{ x: 0, opacity: 1 }}
            exit={{ x: 320, opacity: 0 }}
            transition={{ type: 'spring', damping: 25, stiffness: 200 }}
            className="fixed right-0 top-0 bottom-0 w-80 bg-white shadow-2xl z-40 flex flex-col"
          >
            {/* 头部 */}
            <div className="flex items-center justify-between p-4 border-b border-gray-200">
              <div className="flex items-center gap-2">
                <span className="text-xl">🐱</span>
                <h2 className="font-semibold text-gray-800">番茄猫</h2>
              </div>
              <button
                onClick={() => setIsOpen(false)}
                className="p-1 text-gray-400 hover:text-gray-600 transition-colors"
              >
                <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            {/* Tab 切换 */}
            <div className="flex border-b border-gray-200">
              {(['chat', 'tasks', 'settings'] as const).map((tab) => (
                <button
                  key={tab}
                  onClick={() => setActiveTab(tab)}
                  className={`flex-1 py-3 text-sm font-medium transition-colors relative ${
                    activeTab === tab
                      ? 'text-blue-600 border-b-2 border-blue-600'
                      : 'text-gray-500 hover:text-gray-700'
                  }`}
                >
                  {tab === 'chat' && '💬 聊天'}
                  {tab === 'tasks' && (
                    <>
                      📋 任务
                      {pendingTasks > 0 && (
                        <span className="absolute -top-1 -right-1 bg-red-500 text-white text-xs rounded-full w-4 h-4 flex items-center justify-center">
                          {pendingTasks}
                        </span>
                      )}
                    </>
                  )}
                  {tab === 'settings' && '⚙️ 设置'}
                </button>
              ))}
            </div>

            {/* 内容区域 */}
            <div className="flex-1 overflow-hidden">
              {activeTab === 'chat' && <ChatPanel />}
              {activeTab === 'tasks' && (
                <TaskPanel />
              )}
              {activeTab === 'settings' && (
                <div className="p-4 space-y-5 overflow-y-auto h-full text-sm">
                  {/* ===== 通用 ===== */}
                  <Section title="通用">
                    <SettingRow label="语言">
                      <select
                        value={editSettings?.language ?? 'zh'}
                        onChange={(e) => updateSetting('language', e.target.value as 'zh' | 'en')}
                        className="px-2 py-1 border border-gray-300 rounded text-sm"
                      >
                        <option value="zh">中文</option>
                        <option value="en">English</option>
                      </select>
                    </SettingRow>
                    <SettingRow label="开机自启">
                      <Toggle
                        checked={editSettings?.launchOnStartup ?? false}
                        onChange={(v) => updateSetting('launchOnStartup', v)}
                      />
                    </SettingRow>
                  </Section>

                  {/* ===== 窗口 ===== */}
                  <Section title="窗口">
                    <SettingRow label="透明度">
                      <input
                        type="range"
                        min="0.3"
                        max="1"
                        step="0.05"
                        value={editSettings?.windowOpacity ?? 1}
                        onChange={(e) => updateSetting('windowOpacity', parseFloat(e.target.value))}
                        className="w-24"
                      />
                      <span className="text-xs text-gray-400 w-8 text-right">
                        {Math.round((editSettings?.windowOpacity ?? 1) * 100)}%
                      </span>
                    </SettingRow>
                  </Section>

                  {/* ===== 计时器 ===== */}
                  <Section title="计时器">
                    <SettingRow label="专注时长（分）">
                      <NumberInput
                        value={editSettings?.focusDuration ?? 25}
                        min={1}
                        max={120}
                        onChange={(v) => updateSetting('focusDuration', v)}
                      />
                    </SettingRow>
                    <SettingRow label="短休息（分）">
                      <NumberInput
                        value={editSettings?.shortBreakDuration ?? 5}
                        min={1}
                        max={30}
                        onChange={(v) => updateSetting('shortBreakDuration', v)}
                      />
                    </SettingRow>
                    <SettingRow label="长休息（分）">
                      <NumberInput
                        value={editSettings?.longBreakDuration ?? 15}
                        min={1}
                        max={60}
                        onChange={(v) => updateSetting('longBreakDuration', v)}
                      />
                    </SettingRow>
                  </Section>

                  {/* ===== 进程监控 ===== */}
                  <Section title="进程监控">
                    <SettingRow label="启用分心检测">
                      <Toggle
                        checked={editSettings?.monitorEnabled ?? true}
                        onChange={(v) => updateSetting('monitorEnabled', v)}
                      />
                    </SettingRow>
                    <SettingRow label="冷却时间（秒）">
                      <NumberInput
                        value={editSettings?.monitorCooldown ?? 30}
                        min={5}
                        max={300}
                        onChange={(v) => updateSetting('monitorCooldown', v)}
                      />
                    </SettingRow>
                  </Section>

                  {/* ===== AI ===== */}
                  <Section title="AI">
                    <SettingRow label="Claude Code CLI">
                      <span className={`inline-block w-2 h-2 rounded-full ${
                        claudeStatus === '已检测' ? 'bg-green-500' : 'bg-yellow-500'
                      }`} />
                      <span className="text-xs text-gray-500 ml-1">{claudeStatus}</span>
                    </SettingRow>
                    <SettingRow label="最大对话轮次">
                      <input
                        type="range"
                        min="1"
                        max="10"
                        value={editSettings?.maxTurns ?? 3}
                        onChange={(e) => updateSetting('maxTurns', Number(e.target.value))}
                        className="w-24"
                      />
                      <span className="text-xs text-gray-400 w-4 text-right">
                        {editSettings?.maxTurns ?? 3}
                      </span>
                    </SettingRow>
                    <SettingRow label="预算上限（USD）">
                      <input
                        type="number"
                        min="0"
                        max="1"
                        step="0.01"
                        value={editSettings?.maxBudgetUsd ?? ''}
                        onChange={(e) =>
                          updateSetting('maxBudgetUsd', e.target.value ? parseFloat(e.target.value) : null)
                        }
                        placeholder="不限"
                        className="w-20 px-2 py-1 border border-gray-300 rounded text-sm"
                      />
                      {editSettings?.maxBudgetUsd === null && (
                        <span className="text-xs text-gray-400 ml-1">不限</span>
                      )}
                    </SettingRow>
                    <button
                      onClick={async () => {
                        try {
                          await clearAiSession();
                          alert('✅ AI 对话会话已重置');
                        } catch {
                          alert('❌ 重置失败');
                        }
                      }}
                      className="w-full py-1.5 text-xs text-gray-500 border border-gray-300 rounded-lg hover:bg-gray-100 transition-colors"
                    >
                      🔄 重置 AI 对话会话
                    </button>
                  </Section>

                  {/* 保存按钮 */}
                  <button
                    onClick={handleSave}
                    disabled={isSaving || isLoading}
                    className="w-full py-2 rounded-lg text-sm font-medium bg-blue-500 text-white hover:bg-blue-600 disabled:opacity-50 transition-colors"
                  >
                    {isSaving ? '保存中...' : '💾 保存设置'}
                  </button>
                </div>
              )}
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* 遮罩层 */}
      <AnimatePresence>
        {isOpen && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 bg-black/20 z-30"
            onClick={() => setIsOpen(false)}
          />
        )}
      </AnimatePresence>
    </>
  );
}

// ==================== 小组件 ====================

/** 设置分组标题 */
function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div>
      <h4 className="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-2">{title}</h4>
      <div className="space-y-2">{children}</div>
    </div>
  );
}

/** 设置行：标签 + 控件 */
function SettingRow({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex items-center justify-between py-1">
      <span className="text-gray-600 text-sm">{label}</span>
      <div className="flex items-center gap-2">{children}</div>
    </div>
  );
}

/** 开关组件 */
function Toggle({ checked, onChange }: { checked: boolean; onChange: (v: boolean) => void }) {
  return (
    <button
      onClick={() => onChange(!checked)}
      className={`relative w-10 h-5 rounded-full transition-colors ${
        checked ? 'bg-blue-500' : 'bg-gray-300'
      }`}
    >
      <span
        className={`absolute top-0.5 left-0.5 w-4 h-4 bg-white rounded-full shadow transition-transform ${
          checked ? 'translate-x-5' : ''
        }`}
      />
    </button>
  );
}

/** 数字输入 */
function NumberInput({
  value,
  min,
  max,
  onChange,
}: {
  value: number;
  min: number;
  max: number;
  onChange: (v: number) => void;
}) {
  return (
    <input
      type="number"
      min={min}
      max={max}
      value={value}
      onChange={(e) => onChange(Math.max(min, Math.min(max, parseInt(e.target.value) || min)))}
      className="w-16 px-2 py-1 border border-gray-300 rounded text-sm text-right"
    />
  );
}
