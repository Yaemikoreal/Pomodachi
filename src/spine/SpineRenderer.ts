import { resolveAnimation, IDLE_ANIMATION } from './emotion-map';
import type { RenderHints } from './avatar-config';

export interface SpinePlayerConfig {
  containerId: string;
  skelUrl: string;
  atlasUrl: string;
  renderHints?: RenderHints;
  renderFix?: boolean;
  onLoaded?: () => void;
  onError?: (msg: string) => void;
}

const BASE_WIDTH = 420;
const BASE_HEIGHT = 420;

export class SpineRenderer {
  public player: any;
  private cfg: SpinePlayerConfig;
  private scaleFactor = 1.0;

  constructor(cfg: SpinePlayerConfig) {
    this.cfg = cfg;
    this.createPlayer(cfg.renderHints, cfg.renderFix);
  }

  private createPlayer(renderHints?: RenderHints, renderFix?: boolean) {
    this.destroy();
    const pma = renderHints?.premultipliedAlpha ?? true;
    // 渲染修复开启时，尝试非 PMA 或 bleed 补偿以消除白边
    const effectivePma = renderFix ? !pma : pma;
    // @ts-ignore spine is loaded globally from index.html
    this.player = new spine.SpinePlayer(this.cfg.containerId, {
      skelUrl: this.cfg.skelUrl,
      atlasUrl: this.cfg.atlasUrl,
      animation: IDLE_ANIMATION,
      showControls: false,
      backgroundColor: '#00000000',
      alpha: true,
      premultipliedAlpha: effectivePma,
      loop: true,
      success: () => {
        this.cfg.onLoaded?.();
      },
      error: (_player: any, msg: string) => {
        this.cfg.onError?.(msg);
      },
    });
  }

  /** 记录整体缩放比例，用于命中检测坐标转换 */
  setScale(factor: number) {
    this.scaleFactor = Math.max(0.5, Math.min(1.5, factor));
  }

  /** 应用/取消渲染修复，需要重建 SpinePlayer 以切换 premultipliedAlpha */
  applyRenderFix(enabled: boolean, renderHints?: RenderHints) {
    this.createPlayer(renderHints, enabled);
  }

  /** 基于 Spine SkeletonBounds 的 CPU 命中检测
   *  将屏幕坐标转换为 Spine 世界坐标后检测。
   */
  hitTest(clientX: number, clientY: number): boolean {
    if (!this.player?.skeleton || !this.player?.canvas) return false;

    const canvas = this.player.canvas as HTMLCanvasElement;
    const rect = canvas.getBoundingClientRect();
    // 把屏幕坐标转换为 canvas 内 CSS 像素坐标，再除以 scaleFactor 得到原始 canvas 坐标
    const cssX = clientX - rect.left;
    const cssY = clientY - rect.top;
    const rawX = cssX / this.scaleFactor;
    const rawY = cssY / this.scaleFactor;

    // canvas 中心对应 Spine 世界坐标原点（默认 viewport 下）
    const centerX = canvas.width / 2;
    const centerY = canvas.height / 2;
    const worldX = rawX - centerX;
    const worldY = canvas.height - rawY - centerY; // WebGL y 轴向上

    // @ts-ignore
    const bounds = new spine.SkeletonBounds();
    bounds.update(this.player.skeleton, true);
    return bounds.containsPoint(worldX, worldY);
  }

  play(animationName: string, loop = false) {
    if (!this.player) return;
    this.player.setAnimation(animationName, loop);
  }

  playEmotion(tag: string) {
    const anim = resolveAnimation(tag);
    this.play(anim, false);
    // 情绪动画结束后回到 idle
    setTimeout(() => {
      this.play(IDLE_ANIMATION, true);
    }, 2000);
  }

  destroy() {
    if (this.player) {
      this.player.dispose();
      this.player = null;
    }
  }

  static computeWindowSize(scaleFactor: number): { width: number; height: number } {
    return {
      width: Math.round(BASE_WIDTH * scaleFactor),
      height: Math.round(BASE_HEIGHT * scaleFactor),
    };
  }
}
