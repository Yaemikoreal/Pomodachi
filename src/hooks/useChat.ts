import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface ChatMessage {
  id: number;
  role: 'user' | 'assistant' | 'system';
  content: string;
  createdAt: string;
}

export interface AiConfig {
  id: number;
  maxTurns: number;
  maxBudgetUsd: number | null;
  updatedAt: string;
}

export function useChat() {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // 获取聊天历史
  const fetchHistory = useCallback(async (limit?: number) => {
    try {
      const history = await invoke<ChatMessage[]>('get_chat_history', {
        limit: limit || 50,
      });
      // 过滤掉 system 消息，只显示 user 和 assistant
      setMessages(history.filter((m) => m.role !== 'system'));
      setError(null);
    } catch (err) {
      console.error('获取聊天历史失败:', err);
      setError(String(err));
    }
  }, []);

  // 初始化加载历史
  useEffect(() => {
    fetchHistory();
  }, [fetchHistory]);

  // 发送消息
  const sendMessage = useCallback(
    async (content: string) => {
      if (!content.trim() || isLoading) return;

      const userMessage: ChatMessage = {
        id: Date.now(),
        role: 'user',
        content: content.trim(),
        createdAt: new Date().toISOString(),
      };

      // 先添加用户消息到界面
      setMessages((prev) => [...prev, userMessage]);
      setIsLoading(true);
      setError(null);

      try {
        // 调用后端发送消息
        const response = await invoke<string>('send_chat_message', {
          message: content.trim(),
        });

        const assistantMessage: ChatMessage = {
          id: Date.now() + 1,
          role: 'assistant',
          content: response,
          createdAt: new Date().toISOString(),
        };

        // 添加助手回复到界面
        setMessages((prev) => [...prev, assistantMessage]);
      } catch (err) {
        console.error('发送消息失败:', err);
        setError(String(err));

        // 添加错误消息
        const errorMessage: ChatMessage = {
          id: Date.now() + 1,
          role: 'assistant',
          content: '喵... 我现在有点不舒服，等会儿再聊吧 😿',
          createdAt: new Date().toISOString(),
        };
        setMessages((prev) => [...prev, errorMessage]);
      } finally {
        setIsLoading(false);
      }
    },
    [isLoading]
  );

  // 清空聊天历史
  const clearHistory = useCallback(async () => {
    try {
      await invoke('clear_chat_history');
      setMessages([]);
      setError(null);
    } catch (err) {
      console.error('清空聊天历史失败:', err);
      setError(String(err));
    }
  }, []);

  // 获取 AI 配置
  const getAiConfig = useCallback(async (): Promise<AiConfig | null> => {
    try {
      return await invoke<AiConfig>('get_ai_config');
    } catch (err) {
      console.error('获取 AI 配置失败:', err);
      return null;
    }
  }, []);

  // 更新 AI 配置
  const updateAiConfig = useCallback(
    async (maxTurns: number, maxBudgetUsd: number | null = null) => {
      try {
        await invoke('update_ai_config', {
          maxTurns,
          maxBudgetUsd,
        });
        setError(null);
      } catch (err) {
        console.error('更新 AI 配置失败:', err);
        setError(String(err));
        throw err;
      }
    },
    []
  );

  // 清空 AI 会话
  const clearAiSession = useCallback(async () => {
    try {
      await invoke('clear_ai_session');
      setError(null);
    } catch (err) {
      console.error('清空 AI 会话失败:', err);
      setError(String(err));
    }
  }, []);

  return {
    messages,
    isLoading,
    error,
    sendMessage,
    fetchHistory,
    clearHistory,
    clearAiSession,
    getAiConfig,
    updateAiConfig,
  };
}
