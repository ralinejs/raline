import { useStorage } from '@vueuse/core';
import type { UserInfo } from '@raline/api';
import type { Ref } from 'vue';

export const USER_KEY = 'raline_USER';

export type UserInfoRef = Ref<UserInfo | Record<string, never>>;

let userInfoStorage: UserInfoRef | null = null;

export const useUserInfo = (): UserInfoRef =>
  (userInfoStorage ??= useStorage<UserInfo | Record<string, never>>(
    USER_KEY,
    {},
  ));
