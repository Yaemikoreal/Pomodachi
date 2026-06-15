import { useState, useCallback, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { SpinePet } from './components/SpinePet'
import { StatusPanel } from './components/StatusPanel'
import { Sidebar } from './components/Sidebar'
import { ChatBubble } from './components/ChatBubble'
import { AchievementToast } from './components/AchievementToast'
import { useAchievements } from './hooks/useAchievements'
import './App.css'

function App() {
  const [chatBubbleVisible, setChatBubbleVisible] = useState(false);
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const [skinId, setSkinId] = useState('firefly');
  const { newAchievement, clearNewAchievement } = useAchievements();

  // 初始化加载皮肤
  useEffect(() => {
    invoke<string>('get_pet_skin')
      .then(setSkinId)
      .catch((err) => console.error('获取皮肤失败:', err));
  }, []);

  // 监听皮肤切换事件
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    const setup = async () => {
      unlisten = await listen<string>('pet-skin-changed', (event) => {
        setSkinId(event.payload);
      });
    };
    setup();
    return () => {
      unlisten?.();
    };
  }, []);

  const handlePetClick = useCallback(() => {
    setChatBubbleVisible((prev) => !prev);
  }, []);

  const handleOpenSidebar = useCallback(() => {
    setChatBubbleVisible(false);
    setSidebarOpen(true);
  }, []);

  return (
    <div style={{
      width: '100vw',
      height: '100vh',
      background: 'transparent',
      overflow: 'hidden',
      margin: 0,
      padding: 0,
      position: 'relative',
    }}>
      <SpinePet onPetClick={handlePetClick} skinId={skinId} />
      <StatusPanel />
      <Sidebar open={sidebarOpen} onOpenChange={setSidebarOpen} />
      <ChatBubble
        visible={chatBubbleVisible}
        onClose={() => setChatBubbleVisible(false)}
        onOpenSidebar={handleOpenSidebar}
      />
      <AchievementToast
        achievement={newAchievement}
        onClose={clearNewAchievement}
      />
    </div>
  )
}

export default App
