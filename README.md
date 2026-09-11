# Kubernetes SQLite3 Virtual Table Extension

A sqlite3 virtual table extension that can be loaded into a sqlite3 instance
to get data from Kubernetes. Follows kube-rs defaults for instantiating client
with default being to get data from all namespaces, so potentially can load lots
of data by default.

## Testing

```bash
$ cargo build --release
# Start a sqlite session
$ /opt/homebrew/opt/sqlite/bin/sqlite3  :memory
sqlite> .load target/release/libkubernetes_vtab.dylib
sqlite> CREATE VIRTUAL TABLE pods USING kubernetes_vtab(resource='pods');
-- load some data!
sqlite> select * from test_tab ;
╭─────────────────────────────────────────────────────────────────┬─────────────┬─────────┬───────────────╮
│                              name                               │  namespace  │ status  │ restart_count │
╞═════════════════════════════════════════════════════════════════╪═════════════╪═════════╪═══════════════╡
│ arc-systems-gha-runner-scale-set-controller-gha-rs-controlcfchx │ arc-systems │ Running │             1 │
│ raspberrypi-57f574cf-listener                                   │ arc-systems │ Running │             1 │
│ helm-controller-5984cc88cf-6kh9s                                │ flux-system │ Running │             1 │
│ kustomize-controller-d6bb746df-sdthm                            │ flux-system │ Running │             1 │
│ notification-controller-7bc5c97f8d-swpjc                        │ flux-system │ Running │             1 │
│ source-controller-745c67ff9-6jmx7                               │ flux-system │ Running │             1 │
│ coredns-8db54c48d-wj6hd                                         │ kube-system │ Running │             1 │
│ local-path-provisioner-5d9d9885bc-69psk                         │ kube-system │ Running │             1 │
│ metrics-server-786d997795-pr5gs                                 │ kube-system │ Running │             1 │
│ operator-67f649cf9d-vpxnn                                       │ tailscale   │ Running │             1 │
╰─────────────────────────────────────────────────────────────────┴─────────────┴─────────┴───────────────╯
sqlite> select count(*) from test_tab ;
╭──────────╮
│ count(*) │
╞══════════╡
│       10 │
╰──────────╯
```
