export function guidanceForAiErrorCode(code: string): string | null {
  switch (code) {
    case "AI_DISABLED_OR_GATED":
      return "AI features are currently gated. Complete the required preflight steps (health check, evidence, chunks, index) before drafting.";
    case "AI_OLLAMA_UNHEALTHY":
      return "Ollama is not reachable on 127.0.0.1. Start Ollama locally, then retry the health check. No external network is used.";
    case "AI_INDEX_NOT_READY":
      return "The AI index is not ready. Build evidence chunks first, then build the embeddings index.";
    case "AI_EVIDENCE_EMPTY":
      return "No evidence sources are available. Add at least one evidence source (sanitized export, Slack transcript, report MD, or freeform text).";
    case "AI_EVIDENCE_SOURCE_INVALID":
      return "The evidence source is invalid. Check the selected path exists and matches the chosen source type (file vs directory vs paste).";
    case "AI_INDEX_BUILD_FAILED":
      return "Index build failed. Confirm evidence chunks exist, and that your local environment can write to the app data directory.";
    case "AI_EMBEDDINGS_FAILED":
      return "Embeddings failed. Ensure your Ollama instance is healthy and supports the embeddings endpoint and model you selected (for example, nomic-embed-text).";
    case "AI_RETRIEVAL_FAILED":
      return "Retrieval failed. Ensure the index is built and the query is non-empty, then retry.";
    case "AI_CITATION_REQUIRED":
      return "Citations are required. Select at least one evidence chunk and ensure the draft includes citation markers [[chunk:<chunk_id>]].";
    case "AI_CITATION_INVALID":
      return "Citations are invalid. Ensure cited chunk IDs exist and match the selected citation set.";
    case "AI_DRAFT_FAILED":
      return "Drafting failed. Ensure Ollama is healthy and a local model is installed (the app currently defaults to a local llama3 model).";
    default:
      return null;
  }
}

