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

Every non-2xx maps to `CopepodError::Api { status, code, message }`; use `api_status()`,
`api_code()` and `api_details()` (all work on both variants). Two codes carry their body fields as
`CopepodError::ApiWithDetails` (the whole body is `details`):
402 `ai_credits_exhausted` (`remaining_total`, `resets_at`) and 409 `ai_request_replayed`
(`charged_credits`). Other codes: 403 `hosted_ai_disabled` / `ai_model_not_allowed`,
404 `ai_file_not_found`, 429 `ai_rate_limited` / `ai_provider_rate_limited`,
503 `ai_provider_unavailable`, `ai_rate_limiter_unavailable` (the shared rate limiter is down),
`credits_upgrading`, `ai_upgrading`. Servers before copepod ceaaace send the limiter-outage 503
without a code (`api_code()` is `None`), so match on the status 503, not the code, to cover both. Mistral's own 4xx pass through. The credit headers the server
sends on a 402 are not exposed.

## Validation

Before any request: body is a JSON object; `model` is absent, null or a non-empty string;
`ai_chat` rejects `stream: true` (use `ai_chat_stream`); `Idempotency-Key` is 1-160 chars of
`[A-Za-z0-9_.:-]`; `file_id` is `[A-Za-z0-9_-]{1,128}`; `expiry_hours` is 1-24; transcription
`fields` may not hold `model`, `file` or `stream=true`. `ai_embeddings`, `ai_ocr`,
`ai_upload_file` and `ai_transcribe` take an optional `Idempotency-Key` too.

## Streaming

`ai_chat_stream` forces `stream: true` and yields each SSE `data:` JSON chunk, ending at `[DONE]`.
A failure after the stream started arrives as `data: {"error":{"code","message"}}`; it yields one
`Err(CopepodError::Api)` (503 for `ai_provider_unavailable`, 429 for rate limits, else 502) and the
stream ends. A failure before the first byte is returned by the call itself.
A stream that closes cleanly before `[DONE]` yields one `CopepodError::Sse("stream ended before
[DONE]")`. A `CopepodError::Deserialize` mid-stream also means the answer was interrupted: servers before
ceaaace could garble the error event when the provider failed mid-event, newer ones send a clean
error event. Treat both as a cut-off answer, not as bad input.

`ai_realtime_url` points at a route that answers 404 until Copepod C3 is deployed.

## Model list (`get_ai_models`)

`get_ai_models(org, app, collection)` / `client.app(org, app).ai(collection).models()` calls
`GET …/me/ai/models` with the user bearer and returns `AiModelsResponse`:

- `Available(AiModelList)`: `hosted_ai_enabled`, `default_model`, `models`, `retired`. A plan without
  hosted AI is `Available` with `hosted_ai_enabled: false` and empty lists. `hosted_ai_enabled: true`
  can still have empty `models` and `default_model: None` (nothing priced yet): show free-text entry.
- `Unavailable`: the server answered 404, i.e. a node running an image older than the route (rolling
  upgrade). Keep the last list. Match on it; it is not a `CopepodError`. Every 404 maps here, so an
  unknown app also reads as unavailable.
- 401 and other statuses are `Err(CopepodError::Api { .. })`.

All fields are `#[serde(default)]` and the types are `#[non_exhaustive]`, so older and newer servers
decode. `roles` and `cost_tier` are open strings (`chat`, `analysis`, `vision`, `embedding`, `audio`,
`realtime_audio`, `ocr`, `moderation`; `low`, `medium`, `high`): match known values, ignore the rest.
A hosted chat picker should keep models whose `roles` contain `chat`; the list also holds OCR and
audio models. `retired[].replaced_by` is a suggestion: a call naming a retired model runs on
`default_model`, not on `replaced_by`. Prices are never returned.
