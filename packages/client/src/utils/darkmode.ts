import { isString } from './type.js';

const DARK_VARIABLES = `{--raline-white:#000;--raline-light-grey:#666;--raline-dark-grey:#999;--raline-color:#888;--raline-bg-color:#1e1e1e;--raline-bg-color-light:#272727;--raline-bg-color-hover: #444;--raline-border-color:#333;--raline-disable-bg-color:#444;--raline-disable-color:#272727;--raline-bq-color:#272727;--raline-info-bg-color:#272727;--raline-info-color:#666}`;

export const getDarkStyle = (selector?: string | boolean): string =>
  isString(selector)
    ? selector === 'auto'
      ? `@media(prefers-color-scheme:dark){body${DARK_VARIABLES}}`
      : `${selector}${DARK_VARIABLES}`
    : selector === true
      ? `:root${DARK_VARIABLES}`
      : '';
