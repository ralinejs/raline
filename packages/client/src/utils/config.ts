import { decodePath, isLinkHttp, removeEndingSplash } from './path.js';
import {
  DEFAULT_EMOJI,
  DEFAULT_REACTION,
  defaultHighlighter,
  defaultTeXRenderer,
  defaultUploadImage,
  getDefaultSearchOptions,
  getLang,
  getLocale,
  getMeta,
} from '../config/index.js';
import type {
  RalineEmojiInfo,
  RalineEmojiMaps,
  ralineLocale,
  ralineProps,
} from '../typings/index.js';

export interface ralineEmojiConfig {
  tabs: Pick<RalineEmojiInfo, 'name' | 'icon' | 'items'>[];
  map: RalineEmojiMaps;
}

export interface ralineConfig
  extends Required<
    Omit<
      ralineProps,
      | 'emoji'
      | 'imageUploader'
      | 'highlighter'
      | 'texRenderer'
      | 'wordLimit'
      | 'reaction'
      | 'search'
    >
  > {
  locale: ralineLocale;
  wordLimit: [number, number] | false;
  reaction: string[];
  emoji: Exclude<ralineProps['emoji'], boolean | undefined>;
  highlighter: Exclude<ralineProps['highlighter'], true | undefined>;
  imageUploader: Exclude<ralineProps['imageUploader'], true | undefined>;
  texRenderer: Exclude<ralineProps['texRenderer'], true | undefined>;
  search: Exclude<ralineProps['search'], true | undefined>;
}

export const getServerURL = (serverURL: string): string => {
  const result = removeEndingSplash(serverURL);

  return isLinkHttp(result) ? result : `https://${result}`;
};

const getWordLimit = (
  wordLimit: ralineProps['wordLimit'],
): [number, number] | false =>
  Array.isArray(wordLimit) ? wordLimit : wordLimit ? [0, wordLimit] : false;

const fallback = <T = unknown>(
  value: T | boolean | undefined,
  fallback: T,
): T | false =>
  typeof value === 'function' ? value : value === false ? false : fallback;

export const getConfig = ({
  serverURL,

  path = location.pathname,
  lang = typeof navigator === 'undefined' ? 'en-US' : navigator.language,
  locale,
  emoji = DEFAULT_EMOJI,
  meta = ['nick', 'mail', 'link'],
  requiredMeta = [],
  dark = false,
  pageSize = 10,
  wordLimit,
  imageUploader,
  highlighter,
  texRenderer,
  copyright = true,
  login = 'enable',
  search,
  reaction,
  recaptchaV3Key = '',
  turnstileKey = '',
  commentSorting = 'latest',
  ...more
}: ralineProps): ralineConfig => ({
  serverURL: getServerURL(serverURL),
  path: decodePath(path),
  lang: getLang(lang),
  locale: {
    ...getLocale(getLang(lang)),
    ...(typeof locale === 'object' ? locale : {}),
  } as ralineLocale,
  wordLimit: getWordLimit(wordLimit),
  meta: getMeta(meta),
  requiredMeta: getMeta(requiredMeta),
  imageUploader: fallback(imageUploader, defaultUploadImage),
  highlighter: fallback(highlighter, defaultHighlighter),
  texRenderer: fallback(texRenderer, defaultTeXRenderer),
  dark,
  emoji: typeof emoji === 'boolean' ? (emoji ? DEFAULT_EMOJI : []) : emoji,
  pageSize,
  login,
  copyright,
  search:
    search === false
      ? false
      : typeof search === 'object'
        ? search
        : getDefaultSearchOptions(lang),
  recaptchaV3Key,
  turnstileKey,
  reaction: Array.isArray(reaction)
    ? reaction
    : reaction === true
      ? DEFAULT_REACTION
      : [],
  commentSorting,
  ...more,
});
