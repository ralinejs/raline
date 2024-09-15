import type { RecentCommentData } from '@raline/api';
import { getRecentComment } from '@raline/api';

import { useUserInfo } from '../composables/index.js';
import { getRoot } from '../utils/index.js';

export interface ralineRecentCommentsOptions {
  /**
   * raline 服务端地址
   *
   * raline serverURL
   */
  serverURL: string;

  /**
   * 获取最新评论的数量
   *
   * fetch number of latest comments
   */
  count: number;

  /**
   * 需要挂载的元素
   *
   * Element to be mounted
   */
  el?: string | HTMLElement;

  /**
   * 错误提示消息所使用的语言
   *
   * Language of error message
   *
   * @default navigator.language
   */
  lang?: string;
}

export interface ralineRecentCommentsResult {
  /**
   * 评论数据
   *
   * Comment Data
   */
  comments: RecentCommentData[];

  /**
   * 取消挂载挂件
   *
   * Umount widget
   */
  destroy: () => void;
}

export const RecentComments = ({
  el,
  serverURL,
  count,
  lang = navigator.language,
}: ralineRecentCommentsOptions): Promise<ralineRecentCommentsResult> => {
  const userInfo = useUserInfo();
  const root = getRoot(el);
  const controller = new AbortController();

  return getRecentComment({
    serverURL,
    count,
    lang,
    signal: controller.signal,
    token: userInfo.value?.token,
  }).then((comments) => {
    if (root && comments.length) {
      root.innerHTML = `<ul class="wl-recent-list">${comments
        .map(
          (comment) =>
            `<li class="wl-recent-item"><a href="${comment.url}">${comment.nick}</a>：${comment.comment}</li>`,
        )
        .join('')}</ul>`;

      return {
        comments,
        destroy: (): void => {
          controller.abort();
          root.innerHTML = '';
        },
      };
    }

    return {
      comments,
      destroy: (): void => controller.abort(),
    };
  });
};
