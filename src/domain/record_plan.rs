use super::{DnsZoneSnapshot, RecordChange};
use eyre::Context;
use facet::Facet;

/// Full before/after state and exact intended operation for offline review.
/// No Namecheap write adapter exists in this project yet.
#[derive(Facet, Clone, Debug, PartialEq, Eq)]
pub struct RecordPlan {
    pub before: DnsZoneSnapshot,
    pub after: DnsZoneSnapshot,
    pub change: RecordChange,
}

impl RecordPlan {
    /// # Errors
    /// Fails closed for incomplete, non-Namecheap-hosted, ambiguous, or unsupported edits.
    pub fn build(before: DnsZoneSnapshot, change: RecordChange) -> eyre::Result<Self> {
        before.validate()?;
        eyre::ensure!(before.complete, "complete host snapshot required");
        eyre::ensure!(
            before.provider == "namecheap-basicdns",
            "Namecheap host planning requires a Namecheap BasicDNS zone"
        );
        let mut after = before.clone();
        match &change {
            RecordChange::Add(record) => {
                record.validate_target()?;
                eyre::ensure!(record.host_id.is_none(), "new record cannot have a host ID");
                eyre::ensure!(
                    !after
                        .records
                        .iter()
                        .any(|old| old.host.eq_ignore_ascii_case(&record.host)
                            && old.record_type == record.record_type
                            && old.address == record.address),
                    "duplicate record"
                );
                ensure_cname_compatible(&after.records, &record.host, &record.record_type)?;
                after.records.push(record.clone());
            }
            RecordChange::Update {
                host_id,
                replacement,
            } => {
                replacement.validate_target()?;
                let index = unique_index(&after, host_id)?;
                let old = &after.records[index];
                old.validate_target()
                    .wrap_err("existing record is protected or unsupported")?;
                eyre::ensure!(
                    replacement.host_id.is_none()
                        || replacement.host_id.as_deref() == Some(host_id),
                    "replacement host ID mismatch"
                );
                let mut next = replacement.clone();
                next.host_id = Some(host_id.clone());
                after.records.remove(index);
                ensure_cname_compatible(&after.records, &next.host, &next.record_type)?;
                after.records.insert(index, next);
            }
            RecordChange::Delete { host_id } => {
                let index = unique_index(&after, host_id)?;
                after.records[index]
                    .validate_target()
                    .wrap_err("existing record is protected or unsupported")?;
                after.records.remove(index);
            }
        }
        Ok(Self {
            before,
            after,
            change,
        })
    }

    /// Compare every modeled field, including records unrelated to the intended edit.
    /// A live adapter must re-read immediately before any write and re-check authorization.
    #[must_use]
    pub fn matches_current(&self, current: &DnsZoneSnapshot) -> bool {
        &self.before == current
    }
}

fn unique_index(snapshot: &DnsZoneSnapshot, host_id: &str) -> eyre::Result<usize> {
    eyre::ensure!(!host_id.is_empty(), "host ID required");
    let matches: Vec<_> = snapshot
        .records
        .iter()
        .enumerate()
        .filter(|(_, r)| r.host_id.as_deref() == Some(host_id))
        .map(|(i, _)| i)
        .collect();
    eyre::ensure!(matches.len() == 1, "host ID missing or ambiguous");
    Ok(matches[0])
}

fn ensure_cname_compatible(
    records: &[super::DnsRecord],
    host: &str,
    kind: &str,
) -> eyre::Result<()> {
    eyre::ensure!(
        !records
            .iter()
            .any(|r| r.host.eq_ignore_ascii_case(host)
                && (r.record_type == "CNAME" || kind == "CNAME")),
        "CNAME conflicts with existing host record"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::DnsRecord;

    fn row(id: &str, host: &str, record_type: &str, address: &str) -> DnsRecord {
        DnsRecord {
            host_id: Some(id.into()),
            host: host.into(),
            record_type: record_type.into(),
            address: address.into(),
            ttl: 1800,
            mx_pref: None,
        }
    }

    fn zone(provider: &str) -> DnsZoneSnapshot {
        DnsZoneSnapshot {
            domain: "example.test".into(),
            provider: provider.into(),
            complete: true,
            records: vec![
                row("1", "@", "MX", "mail.example.test"),
                row("2", "@", "TXT", "verification=keep"),
                row("3", "_future", "FUTURE", "opaque"),
                row("4", "www", "A", "192.0.2.4"),
            ],
        }
    }

    #[test]
    fn adding_preserves_unrelated_mail_verification_and_unknown_rows() {
        let before = zone("namecheap-basicdns");
        let added = DnsRecord {
            host_id: None,
            host: "docs".into(),
            record_type: "CNAME".into(),
            address: "pages.example.test".into(),
            ttl: 300,
            mx_pref: None,
        };
        let plan = RecordPlan::build(before.clone(), RecordChange::Add(added.clone())).unwrap();
        assert_eq!(&plan.after.records[..before.records.len()], &before.records);
        assert_eq!(plan.after.records.last(), Some(&added));
        assert!(plan.matches_current(&before));
        let mut raced = before;
        raced.records[1].address = "changed".into();
        assert!(!plan.matches_current(&raced));
    }

    #[test]
    fn protected_and_ambiguous_rows_are_not_mutated() {
        let before = zone("namecheap-basicdns");
        for id in ["1", "2", "3"] {
            let _ = RecordPlan::build(before.clone(), RecordChange::Delete { host_id: id.into() })
                .unwrap_err();
        }
        let mut duplicate_id = before;
        duplicate_id.records[3].host_id = Some("1".into());
        let _ = RecordPlan::build(
            duplicate_id,
            RecordChange::Delete {
                host_id: "1".into(),
            },
        )
        .unwrap_err();
    }

    #[test]
    fn non_namecheap_and_incomplete_zones_fail_closed() {
        let change = RecordChange::Delete {
            host_id: "4".into(),
        };
        let _ = RecordPlan::build(zone("azure-dns"), change.clone()).unwrap_err();
        let mut incomplete = zone("namecheap-basicdns");
        incomplete.complete = false;
        let _ = RecordPlan::build(incomplete, change).unwrap_err();
    }
}
