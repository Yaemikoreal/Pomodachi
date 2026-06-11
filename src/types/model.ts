/** model0.json 中的交互区域定义 */
export interface HitArea {
  name: string
  id: string
  motion: string
  width: number
  height: number
  center_x?: number
  center_y?: number
}

/** model0.json 中的动作定义 */
export interface Motion {
  name?: string
  file: string
  choices?: Array<Record<string, unknown>>
  command?: string
  intimacy?: {
    min: number
    max: number
    bonus?: number
  }
  var_floats?: Array<{
    name?: string
    type: number
    code: string
  }>
  speed?: number
  blend_mode?: number
  wrap_mode?: number
  fade_in?: number
  fade_out?: number
}

/** model0.json 中的控制器配置 */
export interface Controllers {
  param_hit: {
    items: Array<{
      id: string
      hit_area: string
      axis: number
      factor: number
    }>
  }
  param_loop: {
    items: Array<{
      name?: string
      ids: string[]
      type: number
      duration?: number
      blend_mode?: number
      time_sync?: boolean
      min_interval?: number
      max_interval?: number
    }>
  }
  key_trigger: Record<string, unknown>
  eye_blink: {
    min_interval: number
    max_interval: number
  }
  lip_sync: {
    gain: number
  }
  mouse_tracking: {
    smooth_time: number
    items: Array<{
      id: string
      min: number
      max: number
      blend_mode?: number
      input: number
    }>
  }
  auto_breath: Record<string, unknown>
  extra_motion: Record<string, unknown>
  accelerometer: Record<string, unknown>
  intimacy_system: {
    max_value: number
    bonus_active: number
    bonus_inactive: number
    bonus_limit: number
    enabled: boolean
  }
  slot_opacity: Record<string, unknown>
  slot_color: Record<string, unknown>
}

/** model0.json 完整结构 */
export interface ModelConfig {
  type: number
  controllers: Controllers
  hit_areas: HitArea[]
  motions: Record<string, Motion[]>
  physics_v2: Record<string, unknown>
  options: {
    id: string
    name: string
    scale_factor: number
    aniso_level: number
    position_x: number
    position_y: number
    tex_fixed: boolean
    tex_type: number
    spine_sdk: string
    shader_type: number
  }
  skeleton: string
  atlases: Array<{
    atlas: string
    tex_names: string[]
    textures: string[]
  }>
}
