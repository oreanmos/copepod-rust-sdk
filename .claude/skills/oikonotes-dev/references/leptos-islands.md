# Islands, routing and page data

## Component kinds

- `#[component]` — SSR-rendered, no hydration, no browser interactivity.
- `#[island]` — hydrated in the browser; props must be serializable.
- `#[server]` — runs on the server, callable from the client over HTTP.

## Island macro pitfalls

1. No `_`-prefixed variables inside `#[island]` functions: the macro doubles the
   underscore and breaks references.
2. Allows go inside the body as `#![allow(unused_variables)]`. An outer
   `#[allow]` before `#[island]` breaks WASM registration.
3. Empty view: `().into_any()`, not `view! { <span></span> }.into_any()`
   (hydration mismatch).
4. Islands render one stable top-level root element, with the same DOM shape on
   SSR and hydrate for the first render.
5. Listeners added in hydrate are removed in `on_cleanup`; no leaked
   `Closure::forget`.
6. `leptos-daisyui` `Button`: `button_type="submit"`, not `attr:r#type="submit"`.

## Page data loading (canonical)

The server streams out of order (`leptos_axum::render_app_to_stream` in
`crates/app/src/main.rs`): a `Resource` inside `<Suspense>` does not block the
HTML response, even for slow Copepod calls. Reference implementation:
`crates/app/src/pages/inbox.rs`.

```rust
#[component]
pub fn MyPage() -> impl IntoView {
    view! {
        <div class="aesthetic-section overflow-hidden">
            <PageHeader title="My Page" />
            <MyPageContent />
        </div>
    }
}

#[component]
fn MyPageContent() -> impl IntoView {
    let refresh = RwSignal::new(0u32); // bump to re-fetch after a mutation
    let data = Resource::new(move || refresh.get(), |_| async move {
        my_server_fn().await.unwrap_or_default()
    });
    view! {
        <Suspense fallback=move || view! { <SkeletonGrid count=3 height="h-28" /> }>
            {move || data.get().map(|items| view! { /* render */ })}
        </Suspense>
    }
}
```

| Pattern | Where | Use for |
|---|---|---|
| `Resource::new` + `<Suspense>` | page content `#[component]` | all server-fetched page data, including Copepod |
| `LocalResource::new` | leaf island | client-only reactive fetches not needed in SSR HTML |
| `Effect` + `spawn_local` under `#[cfg(feature = "client")]` | leaf island | timers, browser APIs, optimistic UI only |

The `Effect + spawn_local` page-island pattern is deprecated for page data.
Converting an old page: promote the wrapper to `#[component]`, extract a
`*Content` component with `Resource` + `Suspense`, keep only genuinely
interactive widgets as leaf islands, re-fetch with a `RwSignal<u32>` revision,
and run `make app-ssr` and `make app-hydrate` after each subtree.

## Islands router navigation

Full design: `docs/plans/archive/2026-06-22-islands-navigation-architecture.md`.

- `OIKO_ISLANDS_ROUTER` (compile-time, read via `option_env!`;
  `ISLANDS_ROUTER_ENABLED` in `crates/app/src/app_shell/document.rs`) enables
  SPA navigation. The Dockerfile sets it in the `build-tools` stage so both the
  WASM and server builds see it, and this is what production runs today.
- The newer islands-preserving coordinator (`ui/navigation.rs`,
  `window.__oikoNavigation`) ships but stays inert unless enabled at runtime:
  `data-oiko-nav-policy` on `<html>` is built from `OIKO_NAV_ENABLED`,
  `OIKO_NAV_ATOMIC`, `OIKO_NAV_PREFETCH`, `OIKO_NAV_EDITOR` and
  `OIKO_NAV_ALLOW` (all off by default), so rollback needs no rebuild.
- **Route content must never be an `#[island]`.** The router's morph
  (`diffRange` in leptos' `islands_routing.js`) skips `<leptos-island>` by tag,
  so a page-level island never updates on navigation: the URL changes, the body
  does not.
- Route outlets (`app`, `public`, `auth`, `settings`, `budget`, `guide`) are
  `data-route-outlet` attributes, asserted by `layouts_declare_route_outlet_markers`.
  The route manifest is `crates/app/tests/fixtures/route_manifest.rs`; add new
  routes there.
- A stateful island survives navigation only when it sits outside the selected
  outlet. Route replacement must dispose lifecycles (`ui/navigation_lifecycle.rs`);
  editor writes are entity- and session-scoped (`ui/editor_tabs/editor_session.rs`,
  `editor_save_queue.rs`). Fetched route scripts are not executed. When
  navigation is unsure, do a full reload, never a partial guess. The
  navigation-off build must stay correct.
- Prefer router `<A>` for internal links. Links inside `<details><summary>` stay
  plain same-origin `<a href>` with no router hooks, `prevent_default` or
  `stop_propagation`.
- Persistent shell UI (sidebar, breadcrumbs, route-gated actions) may take the
  SSR pathname as an initial seed only; afterwards it reads `window.location`
  and resyncs on `oiko-location-changed` (`LOCATION_CHANGE_SCRIPT`).
  `SIDEBAR_ACTIVE_SCRIPT` reapplies `aria-current="page"`. Do not use
  `use_location`/`use_navigate` in shell islands, and there is no
  `leptos:navigate` event.
- Navigation changes need browser coverage that asserts the navigation mode:
  no new `document` request with the router on; a fresh one, without stale
  content, with it off.
