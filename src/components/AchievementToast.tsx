import { motion, AnimatePresence } from 'framer-motion';

interface AchievementToastProps {
  /** 新解锁的成就 */
  achievement: {
    name: string;
    description: string;
    icon: string;
  } | null;
  /** 关闭回调 */
  onClose: () => void;
}

/**
 * 成就解锁 Toast 通知
 * 收到 achievement-unlocked 事件时弹出祝贺动画
 */
export function AchievementToast({ achievement, onClose }: AchievementToastProps) {
  return (
    <AnimatePresence>
      {achievement && (
        <motion.div
          initial={{ opacity: 0, y: 50, scale: 0.8 }}
          animate={{ opacity: 1, y: 0, scale: 1 }}
          exit={{ opacity: 0, y: -20, scale: 0.8 }}
          transition={{
            type: 'spring',
            damping: 20,
            stiffness: 200,
          }}
          style={{
            position: 'fixed',
            bottom: 80,
            left: '50%',
            transform: 'translateX(-50%)',
            zIndex: 200,
            background: 'linear-gradient(135deg, #fbbf24, #f59e0b)',
            borderRadius: 16,
            padding: '12px 20px',
            boxShadow: '0 8px 32px rgba(245, 158, 11, 0.35)',
            display: 'flex',
            alignItems: 'center',
            gap: 12,
            minWidth: 200,
            cursor: 'pointer',
          }}
          onClick={onClose}
        >
          {/* 装饰背景粒子 */}
          <motion.div
            style={{
              position: 'absolute',
              inset: 0,
              borderRadius: 16,
              background: 'radial-gradient(circle at 30% 50%, rgba(255,255,255,0.2), transparent)',
              pointerEvents: 'none',
            }}
            animate={{ opacity: [0.5, 1, 0.5] }}
            transition={{ duration: 2, repeat: Infinity }}
          />

          {/* 图标 */}
          <motion.span
            style={{ fontSize: 32, lineHeight: 1 }}
            animate={{ rotate: [0, -10, 10, -10, 0] }}
            transition={{ duration: 0.6, delay: 0.2 }}
          >
            {achievement.icon}
          </motion.span>

          {/* 文字 */}
          <div style={{ flex: 1, minWidth: 0 }}>
            <div
              style={{
                fontSize: 12,
                fontWeight: 600,
                color: 'rgba(0,0,0,0.5)',
                marginBottom: 2,
              }}
            >
              🎉 成就解锁
            </div>
            <div
              style={{
                fontSize: 15,
                fontWeight: 700,
                color: '#1a1a1a',
              }}
            >
              {achievement.name}
            </div>
            <div
              style={{
                fontSize: 11,
                color: 'rgba(0,0,0,0.6)',
                marginTop: 1,
              }}
            >
              {achievement.description}
            </div>
          </div>

          {/* 关闭 */}
          <button
            onClick={(e) => {
              e.stopPropagation();
              onClose();
            }}
            style={{
              background: 'rgba(0,0,0,0.1)',
              border: 'none',
              borderRadius: '50%',
              width: 22,
              height: 22,
              fontSize: 12,
              color: 'rgba(0,0,0,0.4)',
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              flexShrink: 0,
            }}
          >
            ✕
          </button>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
