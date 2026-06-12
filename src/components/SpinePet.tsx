import { useEffect, useRef, useState, useCallback } from 'react'
import * as PIXI from 'pixi.js-legacy'
import { Spine, TextureAtlas, AtlasAttachmentLoader, SkeletonBinary } from '@pixi-spine/all-4.1'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { LogicalSize } from '@tauri-apps/api/dpi'
import { invoke } from '@tauri-apps/api/core'
import { usePet, type PetMood } from '../hooks/usePet'

/** mood → 面部动画映射 */
const MOOD_FACE_MAP: Record<PetMood, string> = {
  happy: 'Face_Happy',
  focused: 'Face_Star',
  tired: 'Face_CloseEyesHappy',
  sleeping: 'Face_CloseEyesHappy',
  listening: 'Face_Happy',
  thinking: 'Face_Star',
}

/** 默认身体动画 */
const BODY_IDLE = 'Move_Sit_Idle'

/** 窗口内边距（px） */
const PADDING = 30
/** 最小窗口尺寸 */
const MIN_WIN = 200

interface SpinePetProps {
  onPetClick?: () => void;
}

/**
 * Spine 桌宠渲染组件
 * 渲染后自动裁剪窗口到角色大小，支持拖动和缩放
 */
export function SpinePet({ onPetClick }: SpinePetProps) {
  const { mood } = usePet()
  const containerRef = useRef<HTMLDivElement>(null)
  const appRef = useRef<PIXI.Application | null>(null)
  const spineRef = useRef<Spine | null>(null)
  const [isDragging, setIsDragging] = useState(false)

  /** 将窗口裁剪到贴合精灵 */
  const fitWindowToSpine = useCallback(async (spine: Spine) => {
    try {
      // 触发一帧更新让 bounds 可用
      spine.update(0)
      const bounds = spine.getBounds()

      const w = Math.max(Math.ceil(bounds.width + PADDING), MIN_WIN)
      const h = Math.max(Math.ceil(bounds.height + PADDING), MIN_WIN)

      // 调整 Canvas 大小
      const app = appRef.current
      if (app) {
        app.renderer.resize(w, h)
      }

      // 重定位精灵到居中
      spine.x = -bounds.x + PADDING / 2
      spine.y = -bounds.y + PADDING / 2

      // 调整 Tauri 窗口大小
      const window = getCurrentWindow()
      await window.setSize(new LogicalSize(w, h))

      // 同步容器大小
      if (containerRef.current) {
        containerRef.current.style.width = `${w}px`
        containerRef.current.style.height = `${h}px`
      }
    } catch (e) {
      console.warn('[SpinePet] Bounds fit failed, using skeleton data:', e)
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

  /** 初始化 PixiJS + Spine */
  useEffect(() => {
    if (!containerRef.current) return
    let destroyed = false
    const container = containerRef.current

    const init = async () => {
      try {
        const app = new PIXI.Application({
          width: MIN_WIN,
          height: MIN_WIN,
          backgroundAlpha: 0,
          antialias: true,
          resolution: window.devicePixelRatio || 1,
          autoDensity: true,
          forceCanvas: true,
        })

        if (destroyed) { app.destroy(true); return }

        container.appendChild(app.view as HTMLCanvasElement)
        appRef.current = app

        // 加载 Spine 资源
        const [atlasResponse, textureResponse] = await Promise.all([
          fetch('/spines/firefly/atlases_0_atlas_0'),
          new Promise<HTMLImageElement>((resolve, reject) => {
            const img = new Image()
            img.onload = () => resolve(img)
            img.onerror = reject
            img.src = '/spines/firefly/atlases_0_textures_0_0.png'
          }),
        ])

        if (destroyed || !app.stage) { app.destroy(true); return }
        if (!atlasResponse.ok) throw new Error('Failed to load atlas')
        const atlasText = await atlasResponse.text()

        const skeletonResponse = await fetch('/spines/firefly/skeleton_0')
        if (!skeletonResponse.ok) throw new Error('Failed to load skeleton')
        const skeletonBuffer = await skeletonResponse.arrayBuffer()

        if (destroyed || !app.stage) { app.destroy(true); return }

        // 创建 Spine 资源
        const atlas = new TextureAtlas(atlasText, (_path: string, loaderFunction: (tex: PIXI.BaseTexture) => void) => {
          loaderFunction(new PIXI.BaseTexture(textureResponse))
        })
        const atlasLoader = new AtlasAttachmentLoader(atlas)
        const skeletonLoader = new SkeletonBinary(atlasLoader)
        const skeletonData = skeletonLoader.readSkeletonData(new Uint8Array(skeletonBuffer))

        // 创建 Spine 对象 — 1:1 原始精度（不缩小）
        const spine = new Spine(skeletonData)
        spine.scale.set(1.0)
        spine.x = MIN_WIN / 2
        spine.y = MIN_WIN * 0.85

        // 播放初始动画
        const initialMood = mood ?? 'happy'
        const faceAnim = MOOD_FACE_MAP[initialMood]

        if (skeletonData.findAnimation(BODY_IDLE)) {
          spine.state.setAnimation(0, BODY_IDLE, true)
        } else if (skeletonData.animations.length > 0) {
          spine.state.setAnimation(0, skeletonData.animations[0].name, true)
        }
        if (faceAnim && skeletonData.findAnimation(faceAnim)) {
          spine.state.setAnimation(1, faceAnim, true)
        }

        if (!destroyed && app.stage) {
          // 点击宠物触发聊天气泡
          spine.eventMode = 'static'
          spine.cursor = 'pointer'
          spine.on('pointerdown', () => {
            onPetClick?.()
          })

          app.stage.addChild(spine)
          spineRef.current = spine

          // 等一帧渲染后再裁剪窗口到角色大小
          requestAnimationFrame(() => {
            fitWindowToSpine(spine)
          })
        }
      } catch (e) {
        if (!destroyed) {
          console.error('Failed to init Spine:', e)
          const errDiv = document.createElement('div')
          errDiv.style.cssText =
            'position:absolute;top:10px;left:10px;color:red;font-size:12px;max-width:280px;word-break:break-all;background:rgba(0,0,0,0.5);padding:5px;'
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
  }, []) // eslint-disable-line react-hooks/exhaustive-deps

  // ==================== mood 驱动动画切换 ====================
  useEffect(() => {
    const spine = spineRef.current
    if (!spine?.skeleton?.data) return

    const skeletonData = spine.skeleton.data
    const faceAnim = MOOD_FACE_MAP[mood ?? 'happy']

    // Track 0：身体动画
    if (skeletonData.findAnimation(BODY_IDLE)) {
      const t0 = spine.state.setAnimation(0, BODY_IDLE, true)
      t0.mixDuration = 0.3
    }

    // Track 1：面部表情
    if (faceAnim && skeletonData.findAnimation(faceAnim)) {
      const t1 = spine.state.setAnimation(1, faceAnim, true)
      t1.mixDuration = 0.3
    }
  }, [mood])

  return (
    <>
      <style>{`
        .spine-pet-container:hover .pet-close-btn {
          opacity: 1 !important;
        }
      `}</style>
      <div
        className="spine-pet-container"
        ref={containerRef}
        style={{
          position: 'relative',
          cursor: isDragging ? 'grabbing' : 'grab',
          userSelect: 'none',
          WebkitUserSelect: 'none',
        }}
        onMouseDown={handleMouseDown}
      >
        {/* 关闭按钮 */}
        <button
          onClick={async (e) => {
            e.stopPropagation()
            if (window.confirm('确定退出番茄猫？')) {
              try {
                await invoke('exit_app')
              } catch (err) {
                console.error('退出失败:', err)
              }
            }
          }}
          style={{
            position: 'absolute',
            top: 4,
            left: 4,
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
            zIndex: 20,
            opacity: 0,
            transition: 'opacity 0.2s',
            padding: 0,
          }}
          className="pet-close-btn"
          onMouseEnter={(e) => {
            ;(e.target as HTMLElement).style.background = 'rgba(255,0,0,0.6)'
            ;(e.target as HTMLElement).style.color = 'white'
          }}
          onMouseLeave={(e) => {
            ;(e.target as HTMLElement).style.background = 'rgba(0,0,0,0.3)'
            ;(e.target as HTMLElement).style.color = 'rgba(255,255,255,0.6)'
          }}
        >
          ×
        </button>
      </div>
    </>
  )
}
