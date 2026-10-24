# Namecheap XML source decision

Status (2026-10-24): **synthetic, exact-source qualification only**. The CLI
binary has no XML decoder or live transport. Its production `Cargo.toml` has
not gained an XML dependency.

The owned [`TeamDman/facet-format` source](https://github.com/TeamDman/facet-format/tree/fe271471dfeed52a049ce2df6aeb72bb9c75a632) at
`fe271471dfeed52a049ce2df6aeb72bb9c75a632` provides
`teamy-facet-dom`, `teamy-facet-xml`, and `teamy-facet-xml-node`, all
`0.50.0-rc.7`. It resolves core Facet through the sibling owned
[`TeamDman/facet` source](https://github.com/TeamDman/facet/tree/74b1070dc1c889f429ba79ff458a38dae46209a5)
at `74b1070dc1c889f429ba79ff458a38dae46209a5`. The XML crate pins
`quick-xml =0.41.0` with default features off. The recorded registry archive
SHA-256 is
`e660451e55124f798a69a5af3f49ccfbefbd41910eefd25caf2393e1f3473ec1`.
The [isolated qualification crate](../qualification/owned-facet-xml/README.md)
verifies the sibling source SHAs and runs synthetic Namecheap responses
through the owned strict parser with an injected mock transport.

## What the parser proves

The strict parser bounds response size, nesting, events, attributes, namespace
declarations, and values. It retains expanded names and exact element slices;
the read client retains the full original response. It rejects DTDs, CDATA,
unknown entities, unsupported XML versions/encodings, escaped namespace
declarations, and ambiguous structure. The Namecheap adapter requires the
documented namespace URI on known elements and no namespace on known
attributes. Foreign same-local-name elements and attributes cannot stand in
for a Namecheap `Host` or `Type`. Errors do not expose provider response text.

This is a **read qualification**, not a lossless `setHosts` adapter. Unknown
records and fields are visible and block writes; even a clean response does
not verify Namecheap BasicDNS and email service settings. The current broker
does not approve an exact operation, so no live read/write executor or apply
command is enabled. Namecheap's
[`setHosts` documentation](https://www.namecheap.com/support/api/methods/domains-dns/set-hosts/)
says omitted hosts are removed. A future write must preserve every existing
row and service setting, bind approval to a full snapshot and diff, re-read
before writing, and verify afterward. It cannot claim atomic compare-and-swap.

## Dependency and publication gate

The separate qualification lock deliberately contains both the CLI's
registry `teamy-facet` family and the XML fork's sibling path family at the
same versions. The bridge crosses that boundary only with plain strings and
domain structs; this does **not** establish a portable, unified production
dependency graph. A direct Cargo git dependency on the format fork also is
not self-contained because its core Facet dependencies use `../facet` paths.

The owned format fork's release configuration has workspace `publish=false`
and currently opts in Format, Dessert, JSON, and Value, not this XML family.
Later registry use requires a separate decision to publish and pin, in order,
`teamy-facet-dom` → `teamy-facet-xml` → `teamy-facet-xml-node`, all
`0.50.0-rc.7` or a reviewed compatible next set. DOM depends on local
`facet-singularize 0.50.0-rc.7`; its exact registry availability has not been
established, so that identity must be verified or explicitly replaced/published
before any XML package release. The required owned core/Dessert identities
and registry dependency closure also need validation. No crate publication or
dependency adoption is authorized by this qualification.
