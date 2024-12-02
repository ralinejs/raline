import { useStorage } from '@vueuse/core';

import type { ralineEmojiConfig } from './config.js';
import { removeEndingSplash } from './path.js';
import { isString } from './type.js';
import type { RalineEmojiInfo } from '../typings/index.js';

const hasVersion = (url: string): boolean =>
  Boolean(/@[0-9]+\.[0-9]+\.[0-9]+/.test(url));

const fetchEmoji = (link: string): Promise<RalineEmojiInfo> => {
  const emojiStore = useStorage<Record<string, RalineEmojiInfo>>(
    'raline_EMOJI',
    {},
  );

  const result = hasVersion(link);

  if (result) {
    const info = emojiStore.value[link];

    if (info) return Promise.resolve(info);
  }

  return fetch(`${link}/info.json`)
    .then((resp) => resp.json() as Promise<Omit<RalineEmojiInfo, 'folder'>>)
    .then((emojiInfo) => {
      const info = {
        folder: link,
        ...emojiInfo,
      };

      if (result) emojiStore.value[link] = info;

      return info;
    });
};

const getLink = (name: string, folder = '', prefix = '', type = ''): string =>
  `${folder ? `${folder}/` : ''}${prefix}${name}${type ? `.${type}` : ''}`;

export const getEmojis = (
  emojis: (string | RalineEmojiInfo)[],
): Promise<ralineEmojiConfig> =>
  Promise.all(
    emojis.map((emoji) =>
      isString(emoji)
        ? fetchEmoji(removeEndingSplash(emoji))
        : Promise.resolve(emoji),
    ),
  ).then((emojiInfos) => {
    const emojiConfig: ralineEmojiConfig = {
      tabs: [],
      map: {},
    };

    emojiInfos.forEach((emojiInfo) => {
      const { name, folder, icon, prefix = '', type, items } = emojiInfo;

      emojiConfig.tabs.push({
        name,
        icon: getLink(icon, folder, prefix, type),
        items: items.map((item) => {
          const key = `${prefix}${item}`;

          emojiConfig.map[key] = getLink(item, folder, prefix, type);

          return key;
        }),
      });
    });

    return emojiConfig;
  });
