export interface HitArea {
  name: string;
  id: string;
  motion: string;
  width: number;
  height: number;
  centerX: number;
  centerY: number;
}

export interface RenderHints {
  premultipliedAlpha?: boolean;
  bleed?: boolean;
}

export interface AvatarConfig {
  hitAreas: HitArea[];
  renderHints: RenderHints;
}

export async function loadAvatarConfig(url: string): Promise<AvatarConfig | null> {
  try {
    const res = await fetch(url);
    if (!res.ok) return null;
    const data = await res.json();
    const areas: HitArea[] = (data.hit_areas || []).map((a: any) => ({
      name: a.name,
      id: a.id,
      motion: a.motion,
      width: a.width ?? 1,
      height: a.height ?? 1,
      centerX: a.center_x ?? 0,
      centerY: a.center_y ?? 0,
    }));
    const renderHints: RenderHints = {
      premultipliedAlpha: data.render_hints?.premultipliedAlpha ?? true,
      bleed: data.render_hints?.bleed ?? true,
    };
    return { hitAreas: areas, renderHints };
  } catch (e) {
    console.error('Failed to load avatar config:', e);
    return null;
  }
}
