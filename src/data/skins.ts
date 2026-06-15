export interface SkinInfo {
  id: string;
  name: string;
  description: string;
  thumbnail: string;
}

/** 可用皮肤列表 */
export const AVAILABLE_SKINS: SkinInfo[] = [
  {
    id: 'firefly',
    name: '番茄猫',
    description: '默认皮肤',
    thumbnail: '/spines/firefly/atlases_0_textures_0_0.png',
  },
  // 未来皮肤在此添加：
  // {
  //   id: 'cat2',
  //   name: '另一只猫',
  //   description: '第二款皮肤',
  //   thumbnail: '/spines/cat2/atlases_0_textures_0_0.png',
  // },
];
