import { Marked } from 'marked';
import { markedHighlight } from 'marked-highlight';

import { markedTeXExtensions } from './markedMathExtension.js';
import type {
  ralineEmojiMaps,
  ralineHighlighter,
  ralineTeXRenderer,
} from '../typings/index.js';

export const parseEmoji = (text = '', emojiMap: ralineEmojiMaps = {}): string =>
  text.replace(/:(.+?):/g, (placeholder, key: string) =>
    emojiMap[key]
      ? `<img class="wl-emoji" src="${emojiMap[key]}" alt="${key}">`
      : placeholder,
  );

export interface ParseMarkdownOptions {
  emojiMap: ralineEmojiMaps;
  highlighter: ralineHighlighter | false;
  texRenderer: ralineTeXRenderer | false;
}

export const parseMarkdown = (
  content: string,
  { emojiMap, highlighter, texRenderer }: ParseMarkdownOptions,
): string => {
  const marked = new Marked();

  marked.setOptions({ breaks: true });

  if (highlighter) marked.use(markedHighlight({ highlight: highlighter }));

  if (texRenderer) {
    const extensions = markedTeXExtensions(texRenderer);

    marked.use({ extensions });
  }

  return marked.parse(parseEmoji(content, emojiMap)) as string;
};
