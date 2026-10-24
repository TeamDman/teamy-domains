# Draft: brokered domain operation contract

Status: **design only**. No message in this document is accepted by the current broker, and no Namecheap API call is implemented. This separates an exact domain-operation decision from the current broker's 15-minute 1Password item lease. A credential lease is not permission for any particular read or write. The current broker has a Windows desktop approval dialog; phone approval is not an existing capability.

## Authority boundaries

| Component | Responsibility |
| --- | --- |
| CLI | Parse Figue arguments, construct a typed registrar or DNS request, render typed results. It never accepts an API key or provider endpoint from command arguments. |
| Desktop operation broker (future extension) | Validate the request, own provider credentials and HTTPS client, obtain a one-shot approval for the exact operation, execute it, and return a redacted typed result. It must not return the Namecheap key to the CLI. |
| Credential adapter inside broker | Retrieve the approved Namecheap field from 1Password as needed. Its item lease only permits internal credential use while valid; it does not approve a domain action. |
| Provider adapter | Translate only supported typed operations to documented provider calls. Namecheap registrar delegation and Namecheap BasicDNS records are separate capabilities. |

The caller chooses a provider by typed capability, not by an arbitrary URL or command string. Account/item references and network destination are broker configuration, not a CLI override. Teamy Desktop is the proposed execution location because Namecheap requires an allowlisted public IPv4; no allowlist is configured by this project. No durable cloud credential is needed.

## V1 request shape for review

The implementation should define one versioned, bounded, canonical typed request. Suggested fields are:

```text
operation_request {
  schema_version,
  caller_request_id,             // correlation only; never authority
  provider,                      // e.g. namecheap
  capability,                    // registrar or authoritative_dns
  action,                        // one explicit enum variant
  domain,                        // full exact domain, not a prefix pattern
  expected_state,                // full registrar/zone snapshot for writes
  proposed_state,                // exact intended state for writes
  intended_diff,                 // computed from expected/proposed, never trusted alone
}
```

The broker adds the resolved API destination from its own provider configuration to its internal canonical review record. The caller cannot select or override that endpoint.

Read action variants: `ListDomains`, `GetDomainInfo`, `GetNameservers`, `GetHostRecords`. Write variants: `SetNameservers` and `ReplaceHostRecordsFromPlan`. This first contract excludes purchases, transfers, renewals, DNSSEC changes, arbitrary provider methods and arbitrary HTTP. A later Azure adapter needs record-set semantics and existing Terraform/state ownership resolved before it offers writes. CanSpace account API availability remains unverified.

The broker must reject mismatched provider/capability/domain, unsupported record types or fields, partial snapshots, oversized inputs, and any request that attempts to carry a secret. `GetHostRecords` means the provider's hosted zone; a Namecheap registered domain delegated to Azure DNS does not make Namecheap host rows authoritative. Domain registration metadata and nameserver delegation are registrar data; zone records are DNS-provider data. Website hosting is separate.

## Exact approval and execution

1. The broker validates the typed request and local provider configuration before credential use. A read request obtains a distinct read decision according to an explicit policy. A write starts with an explicit, one-shot **preflight read approval** for the named provider/domain, before the broker accesses the credential. This is not write permission.
2. For a write, the broker then reads the complete current state, checks that it matches the submitted expected state, computes the full before/after diff itself, and presents the provider, domain, exact action, changed rows, preserved rows, and possible outage effect for a separate **write approval**. The broker retains the canonical reviewed request internally. Caller-supplied approval IDs or assertions confer no authority.
3. Approval is one-shot for that exact canonical request and current snapshot. It does not follow the credential lease or authorize later requests with changed domain, type, value, nameservers, or record set. Closing, denial, timeout, cancellation and broker restart fail closed. Human approval time must be realistic; the broker's network and worker deadlines are separate.
4. Immediately before a write, the broker re-reads state and rejects any mismatch. It serializes its own writes per domain, while acknowledging that external writers can still race. For Namecheap `setHosts`, it must prove that every existing host and service setting can be represented in the outgoing full-zone request, preserve unrelated MX, TXT, CAA, NS, verification and other rows, and block if any field cannot be round-tripped. It sends the complete intended zone, then re-reads and verifies it. Namecheap does not document an atomic compare-and-swap for `setHosts`; a successful recheck is not a transaction with other writers.
5. For `SetNameservers`, the broker verifies the target DNS zone and critical records before asking for migration approval, checks delegation again before writing, and verifies provider state after. Public DNS propagation is a later observation, not part of an immediate API success claim. No current nameserver delegation is changed by this design.

Each execution gets a typed outcome: `verified`, `rejected`, or `indeterminate`. A timeout or disconnect after submission must be `indeterminate` until reconciliation reads provider state. Do not blindly retry a whole-zone write or label it a failure. Receipts should contain schema/tool versions, provider operation, exact domain, approved before/after revision, request identity, observation time and verification evidence. They must contain no key, raw authorization header, credential-bearing URL, or unredacted provider error payload.

Suggested typed failures include `approval_denied`, `broker_unavailable`, `unsupported_capability`, `non_authoritative_zone`, `incomplete_snapshot`, `stale_snapshot`, `unsupported_record`, `rate_limited`, `provider_rejected`, `verification_mismatch` and `indeterminate_write`. The current offline planner still reports failures through `eyre`; this enum and its wire encoding remain to implement and test.

## Qualification before live enablement

- Mock a credential adapter and provider adapter; assert that denial, stale snapshot, unsupported fields and cancellation make **zero** write calls.
- Make the displayed approval input and the executed request identical; mutation of any approved field must fail. An identical cached credential lease must not skip approval of a new operation.
- Test full-zone preservation, a second read changed between planning and execution, provider rejection, lost response after a possible write, verification mismatch, and reconciliation. Report the residual external race.
- Test desktop broker availability and its actual approval UI. A phone path needs a separately authenticated, reviewed broker protocol and user interface; the current native dialog supplies no phone claim.
- Review the exact broker SDK/protocol change and any HTTP/XML dependency and pinned graph before adoption. Do not install a new toolchain or dependency to make this offline checkpoint run.

Source basis: [Teamy broker protocol and approval model](https://github.com/TeamDman/teamy-broker/tree/dc129eb74ad791fd41d15ebd830510c3e63ab9ac), [Namecheap API methods](https://www.namecheap.com/support/api/methods/), [Namecheap `getHosts`](https://www.namecheap.com/support/api/methods/domains-dns/get-hosts/), [Namecheap `setHosts`](https://www.namecheap.com/support/api/methods/domains-dns/set-hosts/), [Namecheap API FAQ](https://www.namecheap.com/support/knowledgebase/article.aspx/9739/63/api-faq/), and [Azure DNS record-set API](https://learn.microsoft.com/en-us/rest/api/dns/record-sets?view=rest-dns-2018-05-01).
