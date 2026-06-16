import { useEffect, useRef, useState, useCallback } from 'react'
import * as PIXI from 'pixi.js-legacy'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { LogicalSize } from '@tauri-apps/api/dpi'
import { invoke } from '@tauri-apps/api/core'
import { usePet, type PetMood } from '../hooks/usePet'
import {
  TERMINAL_BASE,
  MOOD_EXPRESSIONS,
  CURSOR_FRAMES,
  COLORS,
  mergeGrids,
  type PixelGrid,
} from '../data/pixel-art'

/** 像素尺寸（每个逻辑像素渲染为 10x10 px） */
const PIXEL_SIZE = 10
/** 网格尺寸 */
const GRID_SIZE = 18
/** 窗口内边距 */
const PADDING = 30
/** 最小窗口尺寸 */
const MIN_WIN = 200
/** 光标闪烁周期（ms） */
const CURSOR_BLINK_INTERVAL = 1000

/** 调试模式 */
const DEBUG = true
const log = (...args: unknown[]) => DEBUG && console.log('[PixelPet]', ...args)

interface PixelPetProps {
  onPetClick?: () => void;
}

export function PixelPet({ onPetClick }: PixelPetProps) {
  const { mood } = usePet()
  const outerRef = useRef<HTMLDivElement>(null)
  const canvasContainerRef = useRef<HTMLDivElement>(null)
  const appRef = useRef<PIXI.Application | null>(null)
  const pixelsRef = useRef<(PIXI.Graphics | null)[][]>([])
  const cursorFrameRef = useRef(0)
  const cursorTimerRef = useRef<ReturnType<typeof setInterval> | null>(null)
  const [isDragging, setIsDragging] = useState(false)

  /** 计算窗口尺寸 */
  const windowWidth = GRID_SIZE * PIXEL_SIZE + PADDING
  const windowHeight = GRID_SIZE * PIXEL_SIZE + PADDING

  /** 将颜色字符串转为十六进制数字 */
  const colorToHex = useCallback((color: string): number => {
    // 处理 #RRGGBB 格式
    if (color.startsWith('#')) {
      return parseInt(color.slice(1), 16)
    }
    return 0x000000
  }, [])

  /** 渲染像素网格 */
  const renderGrid = useCallback((grid: PixelGrid) => {
    for (let y = 0; y < GRID_SIZE; y++) {
      for (let x = 0; x < GRID_SIZE; x++) {
        const pixel = pixelsRef.current[y]?.[x]
        if (!pixel) continue

        const color = grid[y]?.[x]
        if (color) {
          pixel.clear()
          pixel.beginFill(colorToHex(color))
          pixel.drawRect(0, 0, PIXEL_SIZE - 1, PIXEL_SIZE - 1)
          pixel.endFill()
          pixel.visible = true
        } else {
          pixel.visible = false
        }
      }
    }
  }, [colorToHex])

  /** 渲染当前状态（基础 + 表情 + 光标） */
  const renderCurrentState = useCallback(() => {
    const currentMood = mood ?? 'happy'
    const expression = MOOD_EXPRESSIONS[currentMood]

    // 合并基础图案和表情
    let grid = mergeGrids(TERMINAL_BASE, expression)

    // 合并光标帧
    const cursorFrame = CURSOR_FRAMES[cursorFrameRef.current]
    if (cursorFrame) {
      grid = mergeGrids(grid, cursorFrame)
    }

    renderGrid(grid)
  }, [mood, renderGrid])

  /** 鼠标按下拖动 */
  const handleMouseDown = useCallback(async (e: React.MouseEvent) => {
    if (e.button !== 0) return
    setIsDragging(true)
    try {
      await getCurrentWindow().startDragging()
    } catch (err) {
      console.error('[PixelPet] drag:', err)
    } finally {
      setIsDragging(false)
    }
  }, [])

  // ==================== 初始化 PIXI 应用 ====================
  useEffect(() => {
    const canvasCtn = canvasContainerRef.current
    if (!canvasCtn) return
    let destroyed = false

    log('初始化开始')

    // 创建 PIXI Application
    const app = new PIXI.Application({
      width: windowWidth,
      height: windowHeight,
      backgroundAlpha: 0,
      antialias: false,  // 像素风格不需要抗锯齿
      resolution: 1,
    })

    if (destroyed) {
      app.destroy(true)
      return
    }

    canvasCtn.appendChild(app.view as HTMLCanvasElement)
    appRef.current = app
    log('PIXI Application 创建成功')

    // 创建 18x18 像素网格
    for (let y = 0; y < GRID_SIZE; y++) {
      pixelsRef.current[y] = []
      for (let x = 0; x < GRID_SIZE; x++) {
        const pixel = new PIXI.Graphics()
        pixel.beginFill(0x000000)
        pixel.drawRect(0, 0, PIXEL_SIZE - 1, PIXEL_SIZE - 1)
        pixel.endFill()
        pixel.x = x * PIXEL_SIZE + PADDING / 2
        pixel.y = y * PIXEL_SIZE + PADDING / 2
        app.stage.addChild(pixel)
        pixelsRef.current[y][x] = pixel
      }
    }

    log('像素网格创建完成')

    // 初始渲染
    renderCurrentState()

    // 调整 Tauri 窗口尺寸
    getCurrentWindow()
      .setSize(new LogicalSize(windowWidth, windowHeight))
      .catch((e) => console.warn('[PixelPet] window.setSize failed:', e))

    // 同步容器尺寸
    if (outerRef.current) {
      outerRef.current.style.width = `${windowWidth}px`
      outerRef.current.style.height = `${windowHeight}px`
    }

    return () => {
      destroyed = true
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  // ==================== 光标闪烁定时器 ====================
  useEffect(() => {
    // 启动光标闪烁
    cursorTimerRef.current = setInterval(() => {
      cursorFrameRef.current = (cursorFrameRef.current + 1) % CURSOR_FRAMES.length
      renderCurrentState()
    }, CURSOR_BLINK_INTERVAL)

    return () => {
      if (cursorTimerRef.current) {
        clearInterval(cursorTimerRef.current)
        cursorTimerRef.current = null
      }
    }
  }, [renderCurrentState])

  // ==================== mood 变化时重新渲染 ====================
  useEffect(() => {
    log('mood 变化:', mood)
    renderCurrentState()
  }, [mood, renderCurrentState])

  return (
    <>
      <style>{`
        .pixel-pet-container:hover .pet-close-btn { opacity: 1 !important; }
      `}</style>
      <div
        className="pixel-pet-container"
        ref={outerRef}
        style={{
          position: 'relative',
          width: windowWidth,
          height: windowHeight,
          cursor: isDragging ? 'grabbing' : 'grab',
          userSelect: 'none',
          WebkitUserSelect: 'none',
        }}
        onMouseDown={handleMouseDown}
      >
        {/* PIXI Canvas */}
        <div ref={canvasContainerRef} style={{ position: 'absolute', inset: 0, zIndex: 1 }} />

        {/* 关闭按钮 */}
        <button
          onClick={async (e) => {
            e.stopPropagation()
            if (window.confirm('确定退出？')) {
              try {
                await invoke('exit_app')
              } catch {
                // 忽略错误
              }
            }
          }}
          className="pet-close-btn"
          style={{
            position: 'absolute',
            top: 4,
            left: 4,
            zIndex: 20,
            width: 20,
            height: 20,
            borderRadius: '50%',
            border: 'none',
            background: 'rgba(0,0,0,0.3)',
            color: 'rgba(255,255,255,0.6)',
            fontSize: 12,
            lineHeight: '20px',
            textAlign: 'center',
            cursor: 'pointer',
            opacity: 0,
            transition: 'opacity 0.2s',
            padding: 0,
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.background = 'rgba(255,0,0,0.6)'
            e.currentTarget.style.color = 'white'
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.background = 'rgba(0,0,0,0.3)'
            e.currentTarget.style.color = 'rgba(255,255,255,0.6)'
          }}
        >
          ×
        </button>
      </div>
    </>
  )
}
