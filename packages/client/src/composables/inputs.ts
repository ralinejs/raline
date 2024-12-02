import type { RemovableRef } from '@vueuse/core';
import { useStorage } from '@vueuse/core';

export interface UserMeta {
  nick: string;
  mail: string;
  link: string;
}

export const useUserMeta = (): RemovableRef<UserMeta> =>
  useStorage<UserMeta>('raline_USER_META', {
    nick: '',
    mail: '',
    link: '',
  });

export const useEditor = (): RemovableRef<string> =>
  useStorage<string>('raline_COMMENT_BOX_EDITOR', '');
