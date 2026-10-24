# teamy-domains

An offline-first Rust CLI for registrar and authoritative DNS work. This initial checkpoint uses a synthetic snapshot and has **no account, network, credential, broker, or live write adapter**. It cannot change a domain. Namecheap is the first provider whose behavior informs planning; the command model keeps registrars and DNS hosts separate.

Started from [`teamy-rust-cli` at `0dfae8080f780d93bb5b5606050237b6a7b7d478`](https://github.com/TeamDman/teamy-rust-cli/tree/0dfae8080f780d93bb5b5606050237b6a7b7d478). The shared Figue/Facet parser, output renderer, logging, cancellation, paths, version metadata and existing checks were retained. The template-specific `init` command was removed. License: MPL-2.0.

## Offline commands

Every domain command requires `--snapshot PATH` to an explicitly supplied JSON fixture. [`tests/fixtures/example_snapshot.json`](tests/fixtures/example_snapshot.json) is synthetic.

```text
teamy-domains domain list --snapshot tests/fixtures/example_snapshot.json
teamy-domains domain info --snapshot tests/fixtures/example_snapshot.json
teamy-domains nameserver list --snapshot tests/fixtures/example_snapshot.json
teamy-domains nameserver set --snapshot tests/fixtures/example_snapshot.json --nameservers ns1.example.test,ns2.example.test
teamy-domains record list --snapshot tests/fixtures/example_snapshot.json
teamy-domains record add --snapshot tests/fixtures/example_snapshot.json --host docs --record-type CNAME --address pages.example.test --ttl 300
teamy-domains record update --snapshot tests/fixtures/example_snapshot.json --host-id 4 --host www --record-type A --address 192.0.2.5 --ttl 300
teamy-domains record delete --snapshot tests/fixtures/example_snapshot.json --host-id 4
```

`nameserver set` and record add/update/delete return typed **plans**, not applied changes. `--output-format text|json|csv` controls the result. CSV depends on result shape; JSON is the useful plan interchange. `--help` and `--version` work without a snapshot. The first record planner permits A, AAAA and CNAME targets; other records are visible and preserved in the proposed full zone, and targeted edits to MX, TXT and unrecognized types are rejected. It rejects incomplete snapshots, zones other than `namecheap-basicdns`, ambiguous host IDs, and CNAME conflicts. A fixture's `complete` flag is a test assertion, not proof of a live full-zone read. There is deliberately no apply command.

The snapshot has a registrar provider and nameserver delegation independent of its DNS zone provider. A domain registered at Namecheap can use Azure DNS. Its web host is a third, separate decision and is not inferred from either provider.

## Live adapter requirements before enabling it

The current [`teamy-broker` at `dc129eb74ad791fd41d15ebd830510c3e63ab9ac`](https://github.com/TeamDman/teamy-broker/tree/dc129eb74ad791fd41d15ebd830510c3e63ab9ac) approves a 1Password item use and returns a 15 minute secret lease to a same-user client. Identical requests can reuse a lease. It does **not** authorize each domain/API operation, enforce read versus write scopes, or provide phone approval. A future desktop broker executor must take a canonical typed operation, approve the actual domain and before/after diff, execute from the allowlisted desktop IPv4, and return a typed result without exposing the API key to this CLI. Reads also need a deliberate authorization policy. Do not connect the cloud task to a durable Namecheap secret or add `--api-key` arguments.

The [draft exact-operation broker contract](docs/broker-operation-contract.md) records the separate approval, preflight, write-verification, and indeterminate-outcome requirements. It is not implemented by the current broker.

[Namecheap's `setHosts` documentation](https://www.namecheap.com/support/api/methods/domains-dns/set-hosts/) says omitted hosts are deleted. A future mutation path must read the **entire** current zone, preserve all representable rows and service settings, show an exact diff, bind approval to the full snapshot and intended diff, re-read immediately before writing, then verify the returned state. If any field or record type cannot be round-tripped losslessly, block the write. A second writer can still race after recheck because Namecheap does not document an atomic compare-and-swap for this API. Record IDs from `getHosts` are selectors for planning, not stable write preconditions. Mail, DNSSEC, and verification records require explicit preservation checks. Nameserver changes are a separate registrar delegation/migration operation, with independent review of the target zone and its mail/verification records.

Namecheap's [API overview](https://www.namecheap.com/support/api/intro/) and [FAQ](https://www.namecheap.com/support/knowledgebase/article.aspx/9739/63/api-faq/) require enabled API access and an allowlisted IPv4. The FAQ lists production eligibility thresholds and says PremiumDNS/FreeDNS cannot be managed through its API. The intended future execution location is Teamy Desktop's public IPv4; this project neither configures the allowlist nor tests account eligibility. Existing Azure delegation remains as is.

The provider-neutral model does not imply provider support. [Azure DNS exposes record-set APIs](https://learn.microsoft.com/en-us/rest/api/dns/record-sets?view=rest-dns-2018-05-01), but current Terraform state/ownership must be located before any competing write adapter is designed. [CanSpace documents client-area DNS editing](https://canspace.ca/clients/knowledgebase/32/How-do-I-manage-DNS-entries.html); no public account DNS API was established by the source review, so no CanSpace adapter is promised. A registrar inventory adapter must also check each provider's account-specific API capability.

## Dependencies and checks

This checkpoint adds **no dependency** to the copied template graph and performs no dependency or toolchain installation. A live adapter would need an approved broker operation protocol plus reviewed HTTP and XML implementations; existing cached crates are not an adoption decision. The checked repository contains no API credentials, private account fixture, allowlist value, or actual domain inventory.

Run offline tests with the existing toolchain and cached dependencies:

```text
cargo test --offline
cargo clippy --all-targets --all-features --offline -- -D warnings
cargo fmt --all -- --check
```

These tests cover the CLI surface and planning behavior with synthetic data. They do not verify Namecheap responses, broker approval, desktop egress, concurrent live writers, DNS propagation, or any production account.
