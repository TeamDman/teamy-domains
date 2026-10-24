# Owned Facet XML qualification

This separate test crate parses synthetic Namecheap responses with the owned
Facet strict XML source. It is **not** part of the `teamy-domains` binary or its
production dependency graph. Its transport is an in-memory mock; it cannot
contact Namecheap or apply a DNS change.

Place these owned checkouts side by side:

| Checkout | Required HEAD |
| --- | --- |
| `teamy-domains` | this feature branch |
| `facet-format-domains` (`TeamDman/facet-format`) | `fe271471dfeed52a049ce2df6aeb72bb9c75a632` |
| `facet` (`TeamDman/facet`) | `74b1070dc1c889f429ba79ff458a38dae46209a5` |

The `source_pins` test verifies both sibling source SHAs. The harness lockfile
uses `teamy-facet-dom`, `teamy-facet-xml`, and `teamy-facet-xml-node`
`0.50.0-rc.7` from those local checkouts, `facet-singularize 0.50.0-rc.7`
from the owned format checkout, and `quick-xml =0.41.0`. The registry archive
checksum for quick-xml in the owned lockfile is
`e660451e55124f798a69a5af3f49ccfbefbd41910eefd25caf2393e1f3473ec1`.

From the `teamy-domains` root, run:

```sh
cargo test --offline --locked --manifest-path qualification/owned-facet-xml/Cargo.toml
```

The strict parser retains expanded names and exact element slices. The read
client retains the full original response. Parsing errors are mapped to typed,
redacted adapter errors. A parsed `getHosts` result still lacks verified email
and DNS service modes and per-operation broker approval, so it cannot authorize
`setHosts` or make a zone snapshot complete.

No XML family package is currently opted into the owned fork's release config.
Later registry use would require separately reviewing publication of
`teamy-facet-dom`, `teamy-facet-xml`, and `teamy-facet-xml-node` in dependency
order, plus confirming whether the exact `facet-singularize 0.50.0-rc.7`
dependency is available for a portable registry build or making an explicit
owned replacement/publication decision. None of that is done by this harness.
