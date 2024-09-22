import type { RalineComment } from './typings.js';
import type { BaseAPIOptions, ErrorStatusResponse } from './utils.js';
import { errorCheck, getFetchPrefix } from './utils.js';

export interface GetUserListOptions extends BaseAPIOptions {
  /**
   * 每页个数
   *
   * Number per page
   */
  pageSize: number;

  /**
   * 取消请求的信号
   *
   * AbortSignal to cancel request
   */
  signal?: AbortSignal;
}

export interface ralineUser
  extends Pick<RalineComment, 'nick' | 'link' | 'avatar' | 'label' | 'level'> {
  count: number;
}

export interface GetUserListResponse extends ErrorStatusResponse {
  data: ralineUser[];
}

export const getUserList = ({
  serverURL,
  signal,
  pageSize,
  lang,
}: GetUserListOptions): Promise<ralineUser[]> =>
  fetch(`${getFetchPrefix(serverURL)}user?pageSize=${pageSize}&lang=${lang}`, {
    signal,
  })
    .then((resp) => resp.json() as Promise<GetUserListResponse>)
    .then((resp) => errorCheck(resp, 'user list'))
    .then((resp) => resp.data);
