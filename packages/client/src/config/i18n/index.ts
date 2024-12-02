import de from './de.js';
import en from './en.js';
import fr from './fr.js';
import jp from './jp.js';
import ptBR from './pt-BR.js';
import ru from './ru.js';
import viVN from './vi-VN.js';
import zhCN from './zh-CN.js';
import zhTW from './zh-TW.js';
import type { ralineLocale } from '../../typings/index.js';

export type Locales = Record<string, ralineLocale>;

export const DEFAULT_LANG = 'en-US';

export const DEFAULT_LOCALES: Locales = {
  zh: zhCN,
  'zh-cn': zhCN,
  'zh-tw': zhTW,
  en,
  'en-us': en,
  fr,
  'fr-fr': fr,
  jp,
  'jp-jp': jp,
  'pt-br': ptBR,
  ru,
  'ru-ru': ru,
  vi: viVN,
  'vi-vn': viVN,
  de,
};

export const getLocale = (lang: string): ralineLocale =>
  DEFAULT_LOCALES[lang.toLowerCase()] ||
  DEFAULT_LOCALES[DEFAULT_LANG.toLowerCase()];

export const getLang = (lang: string): string =>
  Object.keys(DEFAULT_LOCALES).includes(lang.toLowerCase())
    ? lang
    : DEFAULT_LANG;
