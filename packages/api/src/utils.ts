export interface BaseAPIOptions {
  /**
   * raline 服务端地址
   *
   * raline serverURL
   */
  serverURL: string;

  /**
   * 错误信息所使用的语言
   *
   * Language used in error text
   */
  lang: string;
}

export interface ErrorStatusResponse {
  /**
   * 请求
   */
  instance: string,

  /**
   * 错误标题
   *
   * Error title
   */
  title: string;

  /**
   * 错误消息
   *
   * Error msg detail
   */
  detail: string;
}

export const JSON_HEADERS: Record<string, string> = {
  // eslint-disable-next-line @typescript-eslint/naming-convention
  'Content-Type': 'application/json',
};

export const getFetchPrefix = (serverURL: string): string =>
  `${serverURL.replace(/\/?$/, '/')}api/`;

export const errorCheck = <T extends ErrorStatusResponse>(
  data: T,
  name = '',
): T => {
  if (typeof data === 'object' && data.instance)
    throw new TypeError(`${name} ${data.instance} failed with ${data.title}: ${data.detail}`);

  return data;
};
