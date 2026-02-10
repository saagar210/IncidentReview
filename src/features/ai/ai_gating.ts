export type AiGate = {
  canSearch: boolean;
  canDraft: boolean;
  reasonCode: string | null;
  reasonMessage: string | null;
};

export function computeAiGate(input: {
  healthOk: boolean | null;
  sourcesCount: number;
  chunksCount: number;
  indexReady: boolean | null;
  selectedCitationsCount: number;
}): AiGate {
  if (input.healthOk === false) {
    return {
      canSearch: false,
      canDraft: false,
      reasonCode: "AI_OLLAMA_UNHEALTHY",
      reasonMessage: "Ollama is not reachable on 127.0.0.1.",
    };
  }

  if (input.sourcesCount === 0) {
    return {
      canSearch: false,
      canDraft: false,
      reasonCode: "AI_EVIDENCE_EMPTY",
      reasonMessage: "Add at least one evidence source.",
    };
  }

  if (input.chunksCount === 0) {
    return {
      canSearch: false,
      canDraft: false,
      reasonCode: "AI_INDEX_NOT_READY",
      reasonMessage: "Build evidence chunks before indexing/search.",
    };
  }

  if (input.indexReady === false) {
    return {
      canSearch: false,
      canDraft: false,
      reasonCode: "AI_INDEX_NOT_READY",
      reasonMessage: "Build the embeddings index before searching/drafting.",
    };
  }

  const canSearch = input.healthOk === true && input.sourcesCount > 0 && input.chunksCount > 0 && input.indexReady === true;
  const canDraft = canSearch && input.selectedCitationsCount > 0;

  if (!canDraft) {
    return {
      canSearch,
      canDraft: false,
      reasonCode: "AI_CITATION_REQUIRED",
      reasonMessage: "Select at least one citation chunk before drafting.",
    };
  }

  return {
    canSearch: true,
    canDraft: true,
    reasonCode: null,
    reasonMessage: null,
  };
}
