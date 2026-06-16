import { useEffect, useRef, useState, useCallback } from 'react'
import * as PIXI from 'pixi.js-legacy'
import { Spine, TextureAtlas, AtlasAttachmentLoader, SkeletonBinary } from '@pixi-spine/all-4.1'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { usePet, type PetMood } from '../hooks/usePet'

/** mood → 面部动画映射（参考 agent-pet 的 messageMap 设计） */
const MOOD_FACE_MAP: Record<PetMood, string> = {
  happy: 'Face_Happy',           // 开心微笑
  focused: 'Face_Star',          // 专注星星眼
  tired: 'Face_CloseEyesHappy',  // 疲惫闭眼
  sleeping: 'Face_CloseEyesHappy', // 睡眠（同疲惫）
  listening: 'Face_Happy',       // 聆听（开心脸）
  thinking: 'Face_Star',         // 思考（星星眼）
  error: 'Face_Angry',           // 错误/分心（生气脸）
  waving: 'Face_Happy',          // 挥手打招呼（开心脸）
}

/** mood → 身体动作映射（可选，增加表现力） */
const MOOD_BODY_MAP: Partial<Record<PetMood, string>> = {
  thinking: 'Hand_Ponder',       // 思考动作
  error: 'Hand_Akimbo',          // 叉腰动作
  waving: 'Hand_Clap',           // 拍手/打招呼
}

const BODY_IDLE = 'Move_Sit_Idle'
/** 从 model0.json 取到的 scale_factor */
const MODEL_SCALE = 0.05
/** 宠物目标显示尺寸 */
const PET_SIZE = 150
/** 调试模式 */
const DEBUG = false
const log = (...args: any[]) => DEBUG && console.log('[SpinePet]', ...args)

interface SpinePetProps {
  skinId?: string;
}

export function SpinePet({ skinId = 'firefly' }: SpinePetProps) {
  const { mood } = usePet()
  const outerRef = useRef<HTMLDivElement>(null)
  const canvasContainerRef = useRef<HTMLDivElement>(null)
  const appRef = useRef<PIXI.Application | null>(null)
  const spineRef = useRef<Spine | null>(null)
  const [isDragging, setIsDragging] = useState(false)
  const [isLoading, setIsLoading] = useState(true)
  const [loadError, setLoadError] = useState<string | null>(null)
  const prevSkinRef = useRef(skinId)

  /** 销毁 PIXI 实例 */
  const destroy = useCallback(() => {
    if (appRef.current) {
      appRef.current.destroy(true, { children: true, texture: true, baseTexture: true })
      appRef.current = null
      spineRef.current = null
    }
    if (canvasContainerRef.current) {
      canvasContainerRef.current.innerHTML = ''
    }
  }, [])

  /** 鼠标按下拖动 */
  const handleMouseDown = useCallback(async (e: React.MouseEvent) => {
    if (e.button !== 0) return
    setIsDragging(true)
    try { await getCurrentWindow().startDragging() }
    catch (err) { console.error('[SpinePet] drag:', err) }
    finally { setIsDragging(false) }
  }, [])

  // ==================== 初始化 PIXI + Spine ====================
  useEffect(() => {
    const canvasCtn = canvasContainerRef.current
    if (!canvasCtn) return
    let destroyed = false

    const init = async () => {
      setLoadError(null)
      setIsLoading(true)
      destroy()

      try {
        log('初始化开始, skinId:', skinId)

        // 创建 PIXI Application（使用实际窗口尺寸）
        const winW = window.innerWidth
        const winH = window.innerHeight
        const app = new PIXI.Application({
          width: winW,
          height: winH,
          backgroundAlpha: 0,
          antialias: true,
          resolution: window.devicePixelRatio || 1,
          autoDensity: true,
        })

        if (destroyed) { app.destroy(true); return }

        canvasCtn.appendChild(app.view as HTMLCanvasElement)
        appRef.current = app
        log('PIXI Application 创建成功')

        // 加载 Spine 资产
        const base = `/spines/${skinId}`
        log('加载资源路径:', base)

        const [atlasRes, texImg, skelRes] = await Promise.all([
          fetch(`${base}/atlases_0_atlas_0`),
          new Promise<HTMLImageElement>((resolve, reject) => {
            const img = new Image()
            img.onload = () => {
              log('纹理图片加载成功:', img.width, 'x', img.height)
              resolve(img)
            }
            img.onerror = (e) => {
              log('纹理图片加载失败:', e)
              reject(new Error('纹理图片加载失败'))
            }
            img.src = `${base}/atlases_0_textures_0_0.png`
          }),
          fetch(`${base}/skeleton_0`),
        ])

        if (destroyed || !app.stage) { app.destroy(true); return }
        if (!atlasRes.ok) throw new Error(`图集 HTTP ${atlasRes.status}`)
        if (!skelRes.ok) throw new Error(`骨架 HTTP ${skelRes.status}`)

        const atlasText = await atlasRes.text()
        const skelBuf = await skelRes.arrayBuffer()
        log('Atlas 文本长度:', atlasText.length)
        log('骨架数据大小:', skelBuf.byteLength, 'bytes')

        // 解析图集
        log('开始解析图集...')
        const atlas = new TextureAtlas(atlasText, (path: string, cb: (tex: PIXI.BaseTexture) => void) => {
          log('Atlas 请求纹理路径:', path)
          // 创建纹理
          const texture = PIXI.Texture.from(texImg)
          log('Texture.from 创建成功, width:', texture.width, 'height:', texture.height)
          if (texture.baseTexture) {
            log('BaseTexture 已存在, width:', texture.baseTexture.width, 'height:', texture.baseTexture.height)
            cb(texture.baseTexture)
          } else {
            log('BaseTexture 不存在，创建新的')
            const baseTexture = new PIXI.BaseTexture(texImg)
            cb(baseTexture)
          }
        })
        log('TextureAtlas 解析成功, regions:', atlas.regions.length)

        // 检查 atlas 是否有 region
        if (atlas.regions.length === 0) {
          log('警告: atlas 没有 regions!')
        } else {
          log('第一个 region:', atlas.regions[0].name, 'x:', atlas.regions[0].x, 'y:', atlas.regions[0].y)
        }

        const atlLoader = new AtlasAttachmentLoader(atlas)
        const skelLoader = new SkeletonBinary(atlLoader)
        const skelData = skelLoader.readSkeletonData(new Uint8Array(skelBuf))
        log('骨架数据解析成功, animations:', skelData.animations.length, 'bones:', skelData.bones.length, 'skins:', skelData.skins.length)

        if (destroyed || !app.stage) { app.destroy(true); return }

        // 创建 Spine 对象，先放原点计算 bounds
        const spine = new Spine(skelData)
        spine.scale.set(MODEL_SCALE)
        spine.x = 0
        spine.y = 0

        // 设置渲染属性
        spine.tint = 0xFFFFFF
        spine.blendMode = PIXI.BLEND_MODES.NORMAL

        // 添加到 stage 并更新
        app.stage.addChild(spine)
        spine.update(0)

        // 按 PET_SIZE 等比缩放
        const bounds = spine.getBounds()
        log('Spine 初始 bounds:', bounds.x, bounds.y, bounds.width, bounds.height)
        const fitScale = Math.min(PET_SIZE / bounds.width, PET_SIZE / bounds.height)
        spine.scale.set(MODEL_SCALE * fitScale)
        spine.update(0)

        // 居中：用 pivot 方式精确定位
        const newBounds = spine.getBounds()
        spine.x = winW / 2 - (newBounds.x + newBounds.width / 2)
        spine.y = winH / 2 - (newBounds.y + newBounds.height / 2)

        log('Spine 缩放居中完成, fitScale:', fitScale.toFixed(3), '尺寸:', newBounds.width.toFixed(0), 'x', newBounds.height.toFixed(0), 'pos:', spine.x.toFixed(0), spine.y.toFixed(0))

        // Track 0: 身体待机动画
        if (skelData.findAnimation(BODY_IDLE)) {
          spine.state.setAnimation(0, BODY_IDLE, true)
          log('设置身体动画:', BODY_IDLE)
        } else if (skelData.animations.length > 0) {
          console.warn('[SpinePet]', BODY_IDLE, 'not found, using', skelData.animations[0].name)
          spine.state.setAnimation(0, skelData.animations[0].name, true)
        } else {
          console.warn('[SpinePet] no animations found')
        }

        // Track 1: 面部表情
        const initialMood = mood ?? 'happy'
        const faceAnim = MOOD_FACE_MAP[initialMood]
        if (faceAnim && skelData.findAnimation(faceAnim)) {
          spine.state.setAnimation(1, faceAnim, true)
          log('设置面部动画:', faceAnim)
        }

        // 交互（拖动）
        spine.eventMode = 'static'
        spine.cursor = 'grab'

        spineRef.current = spine
        setIsLoading(false)
        log('Spine 添加到 stage, children:', app.stage.children.length)

        // 启动渲染循环（关键！）
        const tickCallback = () => {
          if (spineRef.current && appRef.current) {
            spineRef.current.update(0)
          }
        }
        app.ticker.add(tickCallback)
        log('PIXI ticker 启动')

      } catch (e) {
        if (!destroyed) {
          const msg = e instanceof Error ? e.message : String(e)
          console.error('[SpinePet] 初始化失败:', msg)
          setLoadError(msg)
          setIsLoading(false)
        }
      }
    }

    // 皮肤切换先淡出再重载
    if (prevSkinRef.current !== skinId) {
      prevSkinRef.current = skinId
      if (outerRef.current) outerRef.current.style.opacity = '0'
      setTimeout(init, 200)
    } else {
      init()
    }

    return () => { destroyed = true }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [skinId])

  // ==================== mood 驱动面部动画和身体动作 ====================
  useEffect(() => {
    const spine = spineRef.current
    if (!spine?.skeleton?.data) return

    const currentMood = mood ?? 'happy'

    // Track 1: 面部表情
    const faceAnim = MOOD_FACE_MAP[currentMood]
    if (spine.skeleton.data.findAnimation(faceAnim)) {
      spine.state.setAnimation(1, faceAnim, true)
      log('设置面部动画:', faceAnim)
    }

    // Track 2: 身体动作（可选）
    const bodyAnim = MOOD_BODY_MAP[currentMood]
    if (bodyAnim && spine.skeleton.data.findAnimation(bodyAnim)) {
      spine.state.setAnimation(2, bodyAnim, false) // 不循环，播放一次
      log('设置身体动作:', bodyAnim)
    }
  }, [mood])

  // ==================== 加载完成后淡入 ====================
  useEffect(() => {
    if (!isLoading && outerRef.current) {
      outerRef.current.style.transition = 'opacity 200ms ease'
      outerRef.current.style.opacity = '1'
    }
  }, [isLoading])

  return (
    <>
      <div
        className="spine-pet-container"
        ref={outerRef}
        style={{
          position: 'relative',
          width: '100vw',
          height: '100vh',
          cursor: isDragging ? 'grabbing' : 'grab',
          userSelect: 'none',
          WebkitUserSelect: 'none',
          opacity: 1,
        }}
        onMouseDown={handleMouseDown}
      >
        {/* PIXI Canvas */}
        <div ref={canvasContainerRef} style={{ position: 'absolute', inset: 0, zIndex: 1 }} />

        {/* 加载中 */}
        {isLoading && !loadError && (
          <div style={{
            position: 'absolute', inset: 0, zIndex: 10,
            display: 'flex', alignItems: 'center', justifyContent: 'center',
            background: 'rgba(0,0,0,0.1)', borderRadius: 8,
          }}>
            <div style={{
              width: 24, height: 24,
              border: '3px solid rgba(255,255,255,0.3)',
              borderTopColor: '#fff',
              borderRadius: '50%',
              animation: 'spin 0.8s linear infinite',
            }} />
            <style>{`@keyframes spin { to { transform: rotate(360deg) } }`}</style>
          </div>
        )}

        {/* 错误提示（隐藏文字，只在控制台输出） */}
        {loadError && (
          <div style={{
            position: 'absolute', top: 4, right: 4, zIndex: 10,
            width: 12, height: 12, borderRadius: '50%',
            background: '#ff6b6b',
            opacity: 0.8,
          }} />
        )}
      </div>
    </>
  )
}
