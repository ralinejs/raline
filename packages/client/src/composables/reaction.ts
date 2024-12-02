import { useStorage } from '@vueuse/core';
import type { Ref } from 'vue';

const REACTION_KEY = 'raline_REACTION';

export type ralineReactionStore = Record<
  /* VOTE_IDENTIFIER */ string,
  number | undefined
>;

export type VoteRef = Ref<ralineReactionStore>;

let reactionStorage: VoteRef | null = null;

export const useReactionStorage = (): VoteRef =>
  (reactionStorage ??= useStorage<ralineReactionStore>(REACTION_KEY, {}));
