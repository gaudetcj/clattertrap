import { describe, expect, it } from 'vitest';
import { familyOf, modelLabel } from './gallery';

describe('endpoint model ids', () => {
  it('are labelled without the vendor, keeping version dots', () => {
    expect(modelLabel('openai-gpt-5.6-luna')).toBe('GPT 5.6 LUNA');
    expect(modelLabel('x-ai-grok-4.3')).toBe('GROK 4.3');
    expect(modelLabel('deepseek-deepseek-v4.1-flash')).toBe('DEEPSEEK V4.1 FLASH');
    expect(modelLabel('z-ai-glm-5.3-flash')).toBe('GLM 5.3 FLASH');
    expect(modelLabel('xiaomi-mimo-v2.6-flash')).toBe('MIMO V2.6 FLASH');
    expect(modelLabel('qwen-qwen3.6-35b-a3b')).toBe('QWEN3.6 35B A3B');
    expect(modelLabel('claude-opus-5-5')).toBe('OPUS 5.5');
  });

  it('belong to a family named by the vendor', () => {
    expect(familyOf('claude-opus-5-5')).toBe('claude');
    expect(familyOf('claude-haiku-4-5')).toBe('claude');
    expect(familyOf('openai-gpt-5.6-luna')).toBe('openai');
    expect(familyOf('x-ai-grok-4.3')).toBe('x-ai');
    expect(familyOf('deepseek-deepseek-v4.1-flash')).toBe('deepseek');
    expect(familyOf('z-ai-glm-5.3-flash')).toBe('z-ai');
    expect(familyOf('xiaomi-mimo-v2.6-flash')).toBe('xiaomi');
    expect(familyOf('qwen-qwen3.6-35b-a3b')).toBe('qwen');
    expect(familyOf('llama3')).toBe('llama3');
  });
});
