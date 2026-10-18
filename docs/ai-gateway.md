# Hosted AI gateway

Methods in `src/api/ai.rs` (also `client.app(org, app).ai(collection)` as `ScopedAiClient`) call the
Copepod AI gateway. Requests and responses are Mistral's own JSON, SSE and multipart shapes passed
through unchanged, so bodies are `serde_json::Value`; the SDK validates only what the server would
reject before calling the provider (body is a JSON object, `model` is a string when present,
`Idempotency-Key` is 1-160 chars of `[A-Za-z0-9_.:-]`, `expiry_hours` is 1-24) and returns
`CopepodError::InvalidArgument` with no network call.

## Auth

| Methods | Route prefix | Auth |
|---|---|---|
| `ai_chat`, `ai_chat_stream`, `ai_embeddings`, `ai_ocr`, `ai_upload_file`, `ai_file_url`, `ai_delete_file`, `ai_transcribe`, `ai_moderations`, `ai_chat_moderations` | `/api/platform/orgs/{org}/apps/{app}/auth/{collection}/me/ai/v1` | app-user bearer |
| `app_ai_moderations` | `/api/platform/orgs/{org}/apps/{app}/ai/v1` | `X-API-Key` only (scope `ai:moderate`); never `Authorization`; optional `X-User-Id` for attribution |

## Mistral-compatible clients

`ai_gateway_base_url(org, app, collection)` returns the route prefix minus `/v1`; point a Mistral
client's base URL there and set its bearer to the app-user access token.
`ai_realtime_url` returns the `ws`/`wss` URL of the realtime transcription route (server slice C3);
the token is not in the URL, send it as a bearer header on the upgrade.

## Idempotency and credits

`ai_chat` and `ai_chat_stream` send `Idempotency-Key` when given (else the gateway uses `gw-{uuid}`).
Reuse a key only to retry the same request: a key that was already charged, or is still running,
answers 409 `ai_request_replayed`. Non-stream methods return `AiResponse<T>` with
`request_id`, `charged_credits`, `remaining_credits` and `estimated` read from the
`X-Copepod-*` headers (each `None`/`false` when absent). Streams are charged after the last byte,
so read `get_current_credits` afterwards.

## Errors

Every non-2xx maps to `CopepodError::Api { status, code, message }`; use `api_status()` and
`api_code()`. Codes: 402 `ai_credits_exhausted`, 403 `hosted_ai_disabled` / `ai_model_not_allowed`,
404 `ai_file_not_found`, 409 `ai_request_replayed`, 429 `ai_rate_limited` /
`ai_provider_rate_limited`, 503 `ai_provider_unavailable` (also when the shared limiter is
unavailable); Mistral's own 4xx pass through. The 402 credit headers are not exposed on the error.

## Streaming

`ai_chat_stream` forces `stream: true` and yields each SSE `data:` JSON chunk, ending at `[DONE]`.
A failure after the stream started arrives as `data: {"error":{"code","message"}}`; it yields one
`Err(CopepodError::Api)` (503 for `ai_provider_unavailable`, 429 for rate limits, else 502) and the
stream ends. A failure before the first byte is returned by the call itself.
