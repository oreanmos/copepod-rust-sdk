# Deployment release, capacity conflicts and status

Requires a Copepod server with deployments reliability (API reference sections
18.4 to 18.7). Older servers: `get_deployment_release` returns a 404 `Api`
error, `allow_outage` is ignored, and the new status fields deserialise to
`None`.

## Release payload

`get_deployment_release(org, deploy)` returns `DeploymentRelease`: `phase`
(`ReleasePhase`), an opaque `version` (compare for equality only),
`runtime_available`, and optional `serving`, `target`, `progress`, `outcome`.
When `runtime_available` is `false` the cluster status was unreadable:
`serving.ready` and `serving.desired` are `None` and only the deploy-log fields
mean anything. Unrecognised phases deserialise to `ReleasePhase::Unknown`. Poll
every 2 s while `Building` or `RollingOut`, otherwise every 15 s.

## Deploy options and capacity conflicts

`deploy_with_options(org, deploy, DeployOptions { mode, allow_outage })` sends
`mode` (default `force`) and `allow_outage: true` only when set. With a single
replica and no room the server answers `409` and queues nothing:

```rust
match client.deploy_with_options(org, id, DeployOptions::default()).await {
    Err(e) if e.api_code() == Some(copepod_sdk::ROLLOUT_NEEDS_OUTAGE_CODE) => {
        let d = e.rollout_capacity_details(); // needed, best_free, startup_probe_seconds...
        // ask the user, then repeat with allow_outage: true
    }
    other => { other?; }
}
```

`rollout_needs_capacity` (blue-green without room) uses the same shape.

## Error accessors

`CopepodError::ApiWithDetails` is used only for the `rollout_needs_outage` and
`rollout_needs_capacity` conflicts. Every other error, including other coded
errors that carry details (record validation, storage quota), stays
`CopepodError::Api`, so existing `Api { status, .. }` matches keep working. Use `api_status()`, `api_code()` and `api_details()` to
handle both. `is_raft_leader_unavailable()` covers both.

## Status

`DeploymentRuntimeStatus` gains optional `serving_replicas`, `rollout`
(`RolloutStatus`) and `release`.
