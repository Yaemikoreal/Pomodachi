import { useEffect, useRef, useState, useCallback } from 'react'
import * as PIXI from 'pixi.js-legacy'
import { Spine, TextureAtlas, AtlasAttachmentLoader, SkeletonBinary } from '@pixi-spine/all-4.1'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { LogicalSize } from '@tauri-apps/api/dpi'
import type { ModelConfig } from '../types/model'

// 缩放配置
const SCALE_LEVELS = [0.5, 0.75, 1, 1.25, 1.5, 2]
const DEFAULT_SCALE_INDEX = 2 // 1x
const BASE_SIZE = 300

/**
 * Spine 桌宠渲染组件
 * 使用 PixiJS Legacy（包含 Canvas 渲染器）渲染 Spine 动画
 * 支持拖动移动和滚轮/键盘缩放
 */
export function SpinePet() {
  const containerRef = useRef<HTMLDivElement>(null)
  const appRef = useRef<PIXI.Application | null>(null)
  const spineRef = useRef<Spine | null>(null)
  const [scaleIndex, setScaleIndex] = useState(DEFAULT_SCALE_INDEX)
  const [isDragging, setIsDragging] = useState(false)

  const scale = SCALE_LEVELS[scaleIndex]

  /** 调整窗口大小 */
  const updateWindowSize = useCallback(async (newScale: number) => {
    try {
      const window = getCurrentWindow()
      const size = BASE_SIZE * newScale
      await window.setSize(new LogicalSize(size, size))
    } catch (err) {
      console.error('调整窗口大小失败:', err)
    }
  }, [])

  /** 鼠标按下 - 开始拖动 */
  const handleMouseDown = useCallback(async (e: React.MouseEvent) => {
    if (e.button !== 0) return

    setIsDragging(true)
    try {
      const window = getCurrentWindow()
      await window.startDragging()
    } catch (err) {
      console.error('拖动失败:', err)
    } finally {
      setIsDragging(false)
    }
  }, [])

  /** 滚轮缩放 */
  const handleWheel = useCallback(async (e: React.WheelEvent) => {
    e.preventDefault()

    let newIndex = scaleIndex
    if (e.deltaY < 0) {
      newIndex = Math.min(scaleIndex + 1, SCALE_LEVELS.length - 1)
    } else {
      newIndex = Math.max(scaleIndex - 1, 0)
    }

    if (newIndex !== scaleIndex) {
      setScaleIndex(newIndex)
      await updateWindowSize(SCALE_LEVELS[newIndex])
    }
  }, [scaleIndex, updateWindowSize])

  /** 键盘快捷键 */
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.ctrlKey || e.metaKey) {
        if (e.key === '=' || e.key === '+') {
          e.preventDefault()
          setScaleIndex(prev => {
            const next = Math.min(prev + 1, SCALE_LEVELS.length - 1)
            updateWindowSize(SCALE_LEVELS[next])
            return next
          })
        } else if (e.key === '-') {
          e.preventDefault()
          setScaleIndex(prev => {
            const next = Math.max(prev - 1, 0)
            updateWindowSize(SCALE_LEVELS[next])
            return next
          })
        } else if (e.key === '0') {
          e.preventDefault()
          setScaleIndex(DEFAULT_SCALE_INDEX)
          updateWindowSize(SCALE_LEVELS[DEFAULT_SCALE_INDEX])
        }
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [updateWindowSize])

  useEffect(() => {
    if (!containerRef.current) return
    let destroyed = false
    const container = containerRef.current

    const init = async () => {
      try {
        // PixiJS Legacy 自动包含 Canvas 渲染器
        const app = new PIXI.Application({
          width: BASE_SIZE,
          height: BASE_SIZE,
          backgroundAlpha: 0,
          antialias: true,
          resolution: window.devicePixelRatio || 1,
          autoDensity: true,
          forceCanvas: true,
        })

        if (destroyed) {
          app.destroy(true)
          return
        }

        // 将 PixiJS 创建的 canvas 添加到容器
        container.appendChild(app.view as HTMLCanvasElement)
        appRef.current = app

        console.log('PixiJS Legacy app created, renderer type:', app.renderer.type === 1 ? 'WebGL' : 'Canvas')

        // 加载 Spine 资源
        const [atlasResponse, textureResponse] = await Promise.all([
          fetch('/spines/firefly/atlases_0_atlas_0'),
          new Promise<HTMLImageElement>((resolve, reject) => {
            const img = new Image()
            img.onload = () => resolve(img)
            img.onerror = reject
            img.src = '/spines/firefly/atlases_0_textures_0_0.png'
          })
        ])

        if (destroyed || !app.stage) {
          app.destroy(true)
          return
        }

        if (!atlasResponse.ok) throw new Error('Failed to load atlas')
        const atlasText = await atlasResponse.text()

        // 读取二进制骨架
        const skeletonResponse = await fetch('/spines/firefly/skeleton_0')
        if (!skeletonResponse.ok) throw new Error('Failed to load skeleton')
        const skeletonBuffer = await skeletonResponse.arrayBuffer()

        if (destroyed || !app.stage) {
          app.destroy(true)
          return
        }

        // 创建 Spine 资源
        const atlas = new TextureAtlas(atlasText, (path: string, loaderFunction: (tex: PIXI.BaseTexture) => void) => {
          const baseTexture = new PIXI.BaseTexture(textureResponse)
          loaderFunction(baseTexture)
        })

        const atlasLoader = new AtlasAttachmentLoader(atlas)
        const skeletonLoader = new SkeletonBinary(atlasLoader)
        const skeletonData = skeletonLoader.readSkeletonData(new Uint8Array(skeletonBuffer))

        // 创建 Spine 对象
        const spine = new Spine(skeletonData)

        // 调整大小和位置
        const spineScale = Math.min(
          BASE_SIZE / (skeletonData.width || 200),
          BASE_SIZE / (skeletonData.height || 200)
        ) * 0.8
        spine.scale.set(spineScale)
        spine.x = BASE_SIZE / 2
        spine.y = BASE_SIZE * 0.93

        // 播放第一个可用动画
        const animations = skeletonData.animations
        console.log('Available animations:', animations.map(a => a.name))
        if (animations.length > 0) {
          spine.state.setAnimation(0, animations[0].name, true)
        }

        if (!destroyed && app.stage) {
          app.stage.addChild(spine)
          spineRef.current = spine
          console.log('Spine pet rendered successfully!')
        }
      } catch (e) {
        if (!destroyed) {
          console.error('Failed to init Spine:', e)
          const errDiv = document.createElement('div')
          errDiv.style.cssText = 'position:absolute;top:10px;left:10px;color:red;font-size:12px;max-width:280px;word-break:break-all;background:rgba(0,0,0,0.5);padding:5px;'
          errDiv.textContent = `Error: ${e instanceof Error ? e.message : String(e)}`
          container.appendChild(errDiv)
        }
      }
    }

    init()

    return () => {
      destroyed = true
      if (appRef.current) {
        appRef.current.destroy(true, { children: true, texture: true, baseTexture: true })
        appRef.current = null
      }
    }
  }, [])

  return (
    <div
      className="spine-pet-container"
      ref={containerRef}
      style={{
        width: `${BASE_SIZE}px`,
        height: `${BASE_SIZE}px`,
        position: 'relative',
        cursor: isDragging ? 'grabbing' : 'grab',
        userSelect: 'none',
        WebkitUserSelect: 'none',
        transform: `scale(${scale})`,
        transformOrigin: 'center center',
      }}
      onMouseDown={handleMouseDown}
      onWheel={handleWheel}
    >
      {/* 缩放指示器 */}
      {scaleIndex !== DEFAULT_SCALE_INDEX && (
        <div
          style={{
            position: 'absolute',
            bottom: 8,
            right: 8,
            background: 'rgba(0,0,0,0.6)',
            color: 'white',
            fontSize: 10,
            padding: '2px 4px',
            borderRadius: 4,
            pointerEvents: 'none',
            zIndex: 10,
          }}
        >
          {Math.round(scale * 100)}%
        </div>
      )}
    </div>
  )
}
