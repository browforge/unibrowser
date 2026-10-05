/** Rough reading speed for status bar (not locale-specific). */
const WORDS_PER_MINUTE = 200;

export interface TextReadingStats {
  words: number;
  minutes: number;
}

export function textReadingStats(text: string): TextReadingStats {
  const trimmed = text.trim();
  if (!trimmed) {
    return { words: 0, minutes: 0 };
  }
  const words = trimmed.split(/\s+/).filter(Boolean).length;
  const minutes = words === 0 ? 0 : Math.max(1, Math.ceil(words / WORDS_PER_MINUTE));
  return { words, minutes };
}
