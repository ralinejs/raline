import { fetchCommentCount } from '@raline/api';

import type { ralineAbort } from './typings/index.js';
import {
  decodePath,
  errorHandler,
  getQuery,
  getServerURL,
} from './utils/index.js';

export interface ralineCommentCountOptions {
  /**
   * raline 服务端地址
   *
   * raline server url
   */
  serverURL: string;

  /**
   * 评论数 CSS 选择器
   *
   * Comment count CSS selector
   *
   * @default '.raline-comment-count'
   */
  selector?: string;

  /**
   * 需要获取的默认路径
   *
   * Path to be fetched by default
   *
   * @default window.location.pathname
   */
  path?: string;

  /**
   * 错误提示消息所使用的语言
   *
   * Language of error message
   *
   * @default navigator.language
   */
  lang?: string;
}

export const commentCount = ({
  serverURL,
  path = window.location.pathname,
  selector = '.raline-comment-count',
  lang = navigator.language,
}: ralineCommentCountOptions): ralineAbort => {
  const controller = new AbortController();

  // comment count
  const elements = document.querySelectorAll<HTMLElement>(selector);

  if (elements.length)
    void fetchCommentCount({
      serverURL: getServerURL(serverURL),
      paths: Array.from(elements).map((element) =>
        decodePath(getQuery(element) ?? path),
      ),
      lang,
      signal: controller.signal,
    })
      .then((counts) => {
        elements.forEach((element, index) => {
          element.innerText = counts[index].toString();
        });
      })
      .catch(errorHandler);

  return controller.abort.bind(controller);
};
