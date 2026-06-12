import { useState, useCallback } from 'react'
import { SpinePet } from './components/SpinePet'
import { StatusPanel } from './components/StatusPanel'
import { Sidebar } from './components/Sidebar'
import { ChatBubble } from './components/ChatBubble'
import './App.css'

function App() {
  const [chatBubbleVisible, setChatBubbleVisible] = useState(false);
  const [sidebarOpen, setSidebarOpen] = useState(false);

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
      <SpinePet onPetClick={handlePetClick} />
      <StatusPanel />
      <Sidebar open={sidebarOpen} onOpenChange={setSidebarOpen} />
      <ChatBubble
        visible={chatBubbleVisible}
        onClose={() => setChatBubbleVisible(false)}
        onOpenSidebar={handleOpenSidebar}
      />
    </div>
  )
}

export default App
