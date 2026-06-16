import { useState, useEffect, useRef, useCallback } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useChat } from '../hooks/useChat';

interface ChatBubbleProps {
  visible: boolean;
  onClose: () => void;
  onOpenSidebar: () => void;
}

export function ChatBubble({ visible, onClose, onOpenSidebar }: ChatBubbleProps) {
  const { messages, isLoading, sendMessage, fetchHistory } = useChat();
  const [input, setInput] = useState('');
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  // 加载最近消息
  useEffect(() => {
    if (visible) {
      fetchHistory(5);
    }
  }, [visible, fetchHistory]);

  // 自动滚动到底部
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  // 聚焦输入框
  useEffect(() => {
    if (visible) {
      setTimeout(() => inputRef.current?.focus(), 100);
    }
  }, [visible]);

  const handleSend = useCallback(async () => {
    const text = input.trim();
    if (!text || isLoading) return;
    setInput('');
    await sendMessage(text);
  }, [input, isLoading, sendMessage]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
    if (e.key === 'Escape') {
      onClose();
    }
  };

  return (
    <AnimatePresence>
      {visible && (
        <motion.div
          initial={{ opacity: 0, scale: 0.9, y: 10 }}
          animate={{ opacity: 1, scale: 1, y: 0 }}
          exit={{ opacity: 0, scale: 0.9, y: 10 }}
          transition={{ duration: 0.2 }}
          style={{
            position: 'fixed',
            bottom: 20,
            left: '50%',
            transform: 'translateX(-50%)',
            width: 280,
            maxHeight: 320,
            background: 'rgba(255,255,255,0.95)',
            backdropFilter: 'blur(12px)',
            borderRadius: 16,
            boxShadow: '0 8px 32px rgba(0,0,0,0.15)',
            display: 'flex',
            flexDirection: 'column',
            zIndex: 100,
            overflow: 'hidden',
          }}
        >
          {/* 头部 */}
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              padding: '8px 12px',
              borderBottom: '1px solid rgba(0,0,0,0.06)',
            }}
          >
            <span style={{ fontSize: 13, fontWeight: 600, color: '#333' }}>
              番茄猫
            </span>
            <div style={{ display: 'flex', gap: 4 }}>
              <button
                onClick={onOpenSidebar}
                title="打开完整聊天"
                style={{
                  background: 'none',
                  border: 'none',
                  fontSize: 14,
                  cursor: 'pointer',
                  padding: '2px 6px',
                  borderRadius: 6,
                  color: '#999',
                }}
              >
                ↗
              </button>
              <button
                onClick={onClose}
                title="关闭"
                style={{
                  background: 'none',
                  border: 'none',
                  fontSize: 14,
                  cursor: 'pointer',
                  padding: '2px 6px',
                  borderRadius: 6,
                  color: '#999',
                }}
              >
                ✕
              </button>
            </div>
          </div>

          {/* 消息列表 */}
          <div
            style={{
              flex: 1,
              overflowY: 'auto',
              padding: '8px 12px',
              minHeight: 60,
              maxHeight: 180,
              scrollbarWidth: 'thin',
            }}
          >
            {messages.length === 0 ? (
              <div style={{ textAlign: 'center', color: '#aaa', fontSize: 12, marginTop: 16 }}>
                和我说点什么吧~
              </div>
            ) : (
              messages.map((msg) => (
                <div
                  key={msg.id}
                  style={{
                    marginBottom: 6,
                    display: 'flex',
                    justifyContent: msg.role === 'user' ? 'flex-end' : 'flex-start',
                  }}
                >
                  <div
                    style={{
                      maxWidth: '80%',
                      padding: '6px 10px',
                      borderRadius: 12,
                      fontSize: 12,
                      lineHeight: 1.4,
                      background: msg.role === 'user' ? '#3b82f6' : '#f0f0f0',
                      color: msg.role === 'user' ? '#fff' : '#333',
                    }}
                  >
                    {msg.content}
                  </div>
                </div>
              ))
            )}
            {isLoading && (
              <div style={{ textAlign: 'center', color: '#999', fontSize: 11, marginTop: 4 }}>
                正在思考...
              </div>
            )}
            <div ref={messagesEndRef} />
          </div>

          {/* 输入区 */}
          <div
            style={{
              display: 'flex',
              gap: 6,
              padding: '8px 12px',
              borderTop: '1px solid rgba(0,0,0,0.06)',
            }}
          >
            <input
              ref={inputRef}
              type="text"
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="说点什么..."
              disabled={isLoading}
              style={{
                flex: 1,
                border: '1px solid #ddd',
                borderRadius: 20,
                padding: '6px 12px',
                fontSize: 12,
                outline: 'none',
              }}
            />
            <button
              onClick={handleSend}
              disabled={isLoading || !input.trim()}
              style={{
                width: 32,
                height: 32,
                borderRadius: '50%',
                border: 'none',
                background: input.trim() ? '#3b82f6' : '#ddd',
                color: '#fff',
                fontSize: 14,
                cursor: input.trim() ? 'pointer' : 'default',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
              }}
            >
              ↑
            </button>
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
