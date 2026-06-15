import { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useTimer, TimerMode } from '../hooks/useTimer';
import { usePet, PetMood } from '../hooks/usePet';
import { usePomodoro } from '../hooks/usePomodoro';
import { useAchievements } from '../hooks/useAchievements';

// 情绪图标映射
const moodEmojis: Record<PetMood, string> = {
  happy: '😺',
  focused: '😼',
  tired: '😿',
  sleeping: '😴',
  listening: '👂',
  thinking: '🤔',
};

// 模式名称映射
const modeNames: Record<TimerMode, string> = {
  focus: '专注',
  shortBreak: '短休息',
  longBreak: '长休息',
};

export function StatusPanel() {
  const timer = useTimer();
  const { mood, petName } = usePet();
  const { todayCount, fetchTodayCount } = usePomodoro();
  const {
    achievements,
    focusStats,
    unlockedCount,
    totalCount,
    formatFocusTime,
    isLoading: achLoading,
  } = useAchievements();
  const [isVisible, setIsVisible] = useState(true);
  const [showAchievements, setShowAchievements] = useState(false);

  // 获取今日专注次数
  useEffect(() => {
    fetchTodayCount();
  }, [fetchTodayCount]);

  // 定时刷新今日专注次数
  useEffect(() => {
    const interval = setInterval(() => {
      fetchTodayCount();
    }, 60000);

    return () => clearInterval(interval);
  }, [fetchTodayCount]);

  return (
    <div className="fixed bottom-4 right-4 z-50">
      {/* 切换按钮 */}
      <motion.button
        className="w-10 h-10 bg-white/90 backdrop-blur-sm rounded-full shadow-lg flex items-center justify-center text-xl mb-2 hover:scale-110 transition-transform"
        whileHover={{ scale: 1.1 }}
        whileTap={{ scale: 0.9 }}
        onClick={() => setIsVisible(!isVisible)}
      >
        {isVisible ? '📊' : '📈'}
      </motion.button>

      {/* 状态面板 */}
      <AnimatePresence>
        {isVisible && (
          <motion.div
            initial={{ opacity: 0, y: 20, scale: 0.9 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: 20, scale: 0.9 }}
            transition={{ duration: 0.3 }}
            className="bg-white/95 backdrop-blur-sm rounded-2xl shadow-xl p-4 w-64"
          >
            {/* 宠物心情 */}
            <div className="flex items-center justify-between mb-3">
              <div className="flex items-center gap-2">
                <span className="text-2xl">{moodEmojis[mood] || '😺'}</span>
                <h3 className="font-semibold text-gray-800">{petName}</h3>
              </div>
              <span className="text-xs text-gray-500">
                🍅 {todayCount} 个番茄
              </span>
            </div>

            {/* 分割线 */}
            <div className="border-t border-gray-200 my-3" />

            {/* 计时器状态 */}
            <div className="mb-3">
              <div className="flex items-center justify-between mb-2">
                <span className="text-sm font-medium text-gray-700">
                  {modeNames[timer.mode]}
                </span>
                <span className="text-xs text-gray-500">
                  🍅 {timer.completedPomodoros}
                </span>
              </div>

              {/* 时间显示 */}
              <div className="text-center mb-2">
                <span className="text-3xl font-mono font-bold text-gray-800">
                  {timer.formattedTime}
                </span>
              </div>

              {/* 进度条 */}
              <div className="h-1.5 bg-gray-200 rounded-full overflow-hidden mb-3">
                <motion.div
                  className={`h-full rounded-full ${
                    timer.mode === 'focus' ? 'bg-red-500' : 'bg-green-500'
                  }`}
                  initial={{ width: 0 }}
                  animate={{ width: `${timer.progress}%` }}
                  transition={{ duration: 0.3 }}
                />
              </div>

              {/* 控制按钮 */}
              <div className="flex justify-center gap-2">
                {!timer.isRunning ? (
                  <motion.button
                    className="px-4 py-1.5 bg-green-500 text-white rounded-full text-sm font-medium hover:bg-green-600 transition-colors"
                    whileHover={{ scale: 1.05 }}
                    whileTap={{ scale: 0.95 }}
                    onClick={timer.start}
                  >
                    ▶ 开始
                  </motion.button>
                ) : (
                  <motion.button
                    className="px-4 py-1.5 bg-yellow-500 text-white rounded-full text-sm font-medium hover:bg-yellow-600 transition-colors"
                    whileHover={{ scale: 1.05 }}
                    whileTap={{ scale: 0.95 }}
                    onClick={timer.pause}
                  >
                    ⏸ 暂停
                  </motion.button>
                )}
                <motion.button
                  className="px-4 py-1.5 bg-gray-500 text-white rounded-full text-sm font-medium hover:bg-gray-600 transition-colors"
                  whileHover={{ scale: 1.05 }}
                  whileTap={{ scale: 0.95 }}
                  onClick={timer.reset}
                >
                  ↺ 重置
                </motion.button>
                <motion.button
                  className="px-4 py-1.5 bg-blue-500 text-white rounded-full text-sm font-medium hover:bg-blue-600 transition-colors"
                  whileHover={{ scale: 1.05 }}
                  whileTap={{ scale: 0.95 }}
                  onClick={timer.skip}
                >
                  ⏭ 跳过
                </motion.button>
              </div>
            </div>

            {/* 分割线 */}
            <div className="border-t border-gray-200 my-3" />

            {/* 今日统计 */}
            <div className="text-center">
              <span className="text-sm text-gray-600">
                今日专注: <span className="font-bold text-green-600">{todayCount}</span> 个番茄
              </span>
            </div>

            {/* 分割线 */}
            <div className="border-t border-gray-200 my-3" />

            {/* 成就概览 */}
            <div>
              <button
                onClick={() => setShowAchievements(!showAchievements)}
                className="w-full flex items-center justify-between text-sm text-gray-600 hover:text-gray-800 transition-colors"
              >
                <span>
                  🏆 成就 ({unlockedCount}/{totalCount})
                </span>
                <span className="text-xs">{showAchievements ? '▲' : '▼'}</span>
              </button>

              <AnimatePresence>
                {showAchievements && (
                  <motion.div
                    initial={{ height: 0, opacity: 0 }}
                    animate={{ height: 'auto', opacity: 1 }}
                    exit={{ height: 0, opacity: 0 }}
                    className="overflow-hidden"
                  >
                    {/* 专注统计 */}
                    {focusStats && (
                      <div className="mt-2 mb-3 space-y-1 text-xs text-gray-500">
                        <div className="flex justify-between">
                          <span>累计专注</span>
                          <span className="font-medium text-gray-700">
                            {formatFocusTime(focusStats.totalFocusSeconds)}
                          </span>
                        </div>
                        <div className="flex justify-between">
                          <span>累计番茄</span>
                          <span className="font-medium text-gray-700">{focusStats.totalPomodoros} 个</span>
                        </div>
                        <div className="flex justify-between">
                          <span>当前连续</span>
                          <span className="font-medium text-orange-600">{focusStats.currentStreak} 天</span>
                        </div>
                        <div className="flex justify-between">
                          <span>最长连续</span>
                          <span className="font-medium text-orange-600">{focusStats.longestStreak} 天</span>
                        </div>
                      </div>
                    )}

                    {/* 成就列表 */}
                    {achLoading ? (
                      <div className="py-2 text-center text-xs text-gray-400">加载中...</div>
                    ) : (
                      <div className="grid grid-cols-3 gap-1.5 pb-2">
                        {achievements.map((ach) => (
                          <div
                            key={ach.id}
                            title={ach.description}
                            className={`flex flex-col items-center p-1.5 rounded-lg text-center ${
                              ach.unlockedAt
                                ? 'bg-amber-50'
                                : 'bg-gray-50 opacity-50'
                            }`}
                          >
                            <span className="text-lg">{ach.icon}</span>
                            <span className="text-[10px] text-gray-500 mt-0.5 leading-tight">
                              {ach.name}
                            </span>
                          </div>
                        ))}
                      </div>
                    )}
                  </motion.div>
                )}
              </AnimatePresence>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
