import { beforeAll, afterAll } from 'vitest';
import { __resetMocks } from './test-utils/mockTauri';

beforeAll(() => {
  // 每次测试套件开始前重置 mocks
  __resetMocks();
});

afterAll(() => {
  __resetMocks();
});
