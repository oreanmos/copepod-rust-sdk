# Storage packs and credit packs

App-user bearer methods on `CopepodClient`, all taking `org_id, app_id, collection`
(`auth/{collection}/me/...`). Never sent with an API key.

## Storage packs (`me/storage-packs*`)

| Method | Call |
|---|---|
| `list_storage_packs` | `GET storage-packs` -> `StoragePackCatalog {storage_packs, owned_storage, capacity_open}` |
| `subscribe_storage_pack(pack_id, quantity, idempotency_key)` | `POST storage-packs/{pack_id}/subscribe` -> `StoragePackSubscription {pack_subscription_id, status, ends_at}` |
| `cancel_storage_pack(subscription_id)` | `DELETE storage-packs/{id}` -> `StoragePackSubscription` (the server names the path parameter `pack_id`; it is the subscription id) |

A pack is a monthly subscription charged on the payment mandate of the user's paid
plan. `subscribe` answers `pending` and adds no space: re-read `list_storage_packs`
until the pack in `owned_storage` is `active`. `quantity` is 1..=100
(`STORAGE_PACK_MAX_QUANTITY`), the key 1-160 ASCII letters, digits, `.:_-`; both are
checked before any request. A retry with the same key returns the same subscription
(even while `capacity_open` is false). Subscribe only when `capacity_open` is true.

`cancel` is idempotent: a paid pack becomes `cancelling` and keeps its space until
`ends_at`; a pack never paid becomes `cancelled` (`ends_at` is now).

### Error codes

Read with `err.api_code()`; constants are exported at the crate root.

| Code | Status | Constant | Meaning |
|---|---|---|---|
| `requires_paid_plan` | 402 | `REQUIRES_PAID_PLAN_CODE` | no active, non-trial paid plan |
| `currency_mismatch` | 409 | `CURRENCY_MISMATCH_CODE` | pack priced in another currency than the plan |
| `storage_unlimited` | 409 | `STORAGE_UNLIMITED_CODE` | plan already has unlimited storage |
| `capacity_unavailable` | 409 | `STORAGE_CAPACITY_UNAVAILABLE_CODE` | app is out of spare storage |
| `mandate_unavailable` | 409 | `MANDATE_UNAVAILABLE_CODE` | no usable mandate; retry later with the same key |
| `payment_provider_rejected` | 502 | `PAYMENT_PROVIDER_REJECTED_CODE` | provider refused the subscription |
| `packs_upgrading` | 503 | `PACKS_UPGRADING_CODE` | rolling upgrade; retry |
| `pack_unavailable` | 404 | | pack not on sale |
| `idempotency_key_reused` | 409 | | key already used for another pack or quantity |

Servers before copepod 397b1889 have no storage pack routes (404).

## Credit packs (`me/credits/packs*`, `me/credits/purchases*`)

`list_packs`, `purchase_credit_pack` (Idempotency-Key, `redirect_url`),
`list_credit_purchases`, `get_credit_purchase`. Credits are added by the server's
payment webhook: poll the purchase until it is `paid`.
