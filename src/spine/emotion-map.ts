// 情绪标签 -> Spine 动画名称
export const EMOTION_ANIMATION_MAP: Record<string, string> = {
  开心: 'Face_Happy',
  害羞: 'Face_CloseEyesHappy',
  认真: 'Hand_Ponder',
  惊讶: 'Face_Star',
  委屈: 'Face_Doubt',
  生气: 'Face_Angry',
  困: 'Move_Sit_Idle',
};

export const IDLE_ANIMATION = 'Move_Sit_Idle';

export function resolveAnimation(tag?: string): string {
  if (!tag) return IDLE_ANIMATION;
  return EMOTION_ANIMATION_MAP[tag] || IDLE_ANIMATION;
}
