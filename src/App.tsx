import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { SpinePet } from './components/SpinePet'
import './App.css'

function App() {
  const [skinId, setSkinId] = useState('firefly');

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
      <SpinePet skinId={skinId} />
    </div>
  )
}

export default App
