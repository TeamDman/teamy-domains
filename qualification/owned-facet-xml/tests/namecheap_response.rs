use std::cell::Cell;

use teamy_domains::domain::{DnsZoneSnapshot, RecordChange, RecordPlan};
use teamy_domains::providers::namecheap::{
    NamecheapAdapterError, NamecheapDomainId, NamecheapReadClient, NamecheapRequestFields,
    NamecheapTransport, NamecheapWriteBlocker, NamecheapXmlDecoder,
};
use teamy_domains_owned_xml_qualification::FacetSourceDecoder;

const UNKNOWN: &str = include_str!("../../../tests/fixtures/namecheap/get_hosts_unknown.xml");
const ERROR: &str = include_str!("../../../tests/fixtures/namecheap/get_hosts_error.xml");
const OK: &str = include_str!("../../../tests/fixtures/namecheap/get_hosts_ok.xml");
const NS: &str = "http://api.namecheap.com/xml.response";

struct MockTransport {
    response: String,
    calls: Cell<usize>,
}

impl NamecheapTransport for MockTransport {
    fn exchange(&self, request: &NamecheapRequestFields) -> Result<String, NamecheapAdapterError> {
        assert_eq!(request.method(), "namecheap.domains.dns.getHosts");
        assert_eq!(
            request.params(),
            &[
                ("SLD".into(), "example".into()),
                ("TLD".into(), "test".into())
            ]
        );
        self.calls.set(self.calls.get() + 1);
        Ok(self.response.clone())
    }
}

fn client(response: impl Into<String>) -> NamecheapReadClient<MockTransport, FacetSourceDecoder> {
    NamecheapReadClient::new(
        MockTransport {
            response: response.into(),
            calls: Cell::new(0),
        },
        FacetSourceDecoder,
    )
}

fn domain() -> NamecheapDomainId {
    NamecheapDomainId::new("example", "test").unwrap()
}

#[test]
fn real_decoder_retains_unknown_fields_and_full_response() {
    let client = client(UNKNOWN);
    let response = client.get_hosts(domain()).unwrap();
    assert_eq!(response.raw_response(), Some(UNKNOWN));
    assert_eq!(response.raw_tree().namespace.as_deref(), Some(NS));
    assert_eq!(
        response
            .raw_tree()
            .raw_xml
            .as_deref()
            .unwrap()
            .starts_with("<ApiResponse"),
        true
    );
    assert_eq!(response.hosts().len(), 1);
    assert_eq!(response.hosts()[0].record_type, "FUTURE");
    assert_eq!(response.hosts()[0].extra_attributes[0].value, "keep");
    assert_eq!(response.extra_elements()[0].text, "preserve me");
    assert!(
        response.extra_elements()[0]
            .raw_xml
            .as_deref()
            .unwrap()
            .contains("<FutureChild Enabled=\"true\">")
    );
    for blocker in [
        NamecheapWriteBlocker::UnknownHostAttribute,
        NamecheapWriteBlocker::UnknownResultAttribute,
        NamecheapWriteBlocker::UnknownResultElement,
        NamecheapWriteBlocker::UnsupportedRecordType,
        NamecheapWriteBlocker::UnverifiedEmailServiceMode,
        NamecheapWriteBlocker::BrokerOperationApprovalUnavailable,
    ] {
        assert!(
            response.write_blockers().contains(&blocker),
            "missing {blocker:?}"
        );
    }
}

#[test]
fn real_decoder_preserves_mail_verification_and_web_rows_in_order() {
    let response = client(OK).get_hosts(domain()).unwrap();
    assert_eq!(response.raw_response(), Some(OK));
    assert_eq!(response.hosts().len(), 3);
    assert_eq!(response.hosts()[0].record_type, "MX");
    assert_eq!(response.hosts()[0].address, "mail.example.test");
    assert_eq!(response.hosts()[0].mx_pref.as_deref(), Some("10"));
    assert_eq!(response.hosts()[0].ttl.as_deref(), Some("1800"));
    assert_eq!(response.hosts()[1].record_type, "TXT");
    assert_eq!(response.hosts()[1].address, "verification=synthetic");
    assert_eq!(response.hosts()[1].ttl.as_deref(), Some("300"));
    assert_eq!(response.hosts()[2].record_type, "A");
    assert_eq!(response.hosts()[2].address, "192.0.2.4");
    assert!(
        response
            .write_blockers()
            .contains(&NamecheapWriteBlocker::UnverifiedEmailServiceMode)
    );
    assert!(
        response
            .write_blockers()
            .contains(&NamecheapWriteBlocker::BrokerOperationApprovalUnavailable)
    );
}

#[test]
fn provider_error_is_typed_and_does_not_log_raw_text() {
    let error = client(ERROR).get_hosts(domain()).unwrap_err();
    assert_eq!(
        error,
        NamecheapAdapterError::ProviderRejected {
            code: Some(2_030_288)
        }
    );
    assert!(!format!("{error:?}").contains("proper DNS servers"));
}

#[test]
fn foreign_namespaces_cannot_impersonate_known_fields() {
    let foreign_host = format!(
        "<ApiResponse xmlns='{NS}' xmlns:x='urn:foreign' Status='OK'><Errors/><RequestedCommand>namecheap.domains.dns.getHosts</RequestedCommand><CommandResponse Type='namecheap.domains.dns.getHosts'><DomainDNSGetHostsResult Domain='example.test' IsUsingOurDNS='true'><x:Host HostId='1' Name='www' Type='A' Address='192.0.2.1'/></DomainDNSGetHostsResult></CommandResponse></ApiResponse>"
    );
    let response = client(foreign_host).get_hosts(domain()).unwrap();
    assert!(response.hosts().is_empty());
    assert_eq!(
        response.extra_elements()[0].namespace.as_deref(),
        Some("urn:foreign")
    );
    assert!(
        response
            .write_blockers()
            .contains(&NamecheapWriteBlocker::UnknownResultElement)
    );

    let foreign_type = format!(
        "<ApiResponse xmlns='{NS}' xmlns:x='urn:foreign' Status='OK'><Errors/><RequestedCommand>namecheap.domains.dns.getHosts</RequestedCommand><CommandResponse x:Type='namecheap.domains.dns.getHosts'><DomainDNSGetHostsResult Domain='example.test' IsUsingOurDNS='true'/></CommandResponse></ApiResponse>"
    );
    assert_eq!(
        client(foreign_type).get_hosts(domain()),
        Err(NamecheapAdapterError::MalformedResponse)
    );

    let foreign_host_attribute = format!(
        "<ApiResponse xmlns='{NS}' xmlns:x='urn:foreign' Status='OK'><Errors/><RequestedCommand>namecheap.domains.dns.getHosts</RequestedCommand><CommandResponse Type='namecheap.domains.dns.getHosts'><DomainDNSGetHostsResult Domain='example.test' IsUsingOurDNS='true'><Host HostId='1' Name='www' Type='A' x:Type='TXT' Address='192.0.2.1'/></DomainDNSGetHostsResult></CommandResponse></ApiResponse>"
    );
    let response = client(foreign_host_attribute).get_hosts(domain()).unwrap();
    assert_eq!(response.hosts()[0].record_type, "A");
    assert_eq!(
        response.hosts()[0].extra_attributes[0].namespace.as_deref(),
        Some("urn:foreign")
    );
    assert_eq!(response.hosts()[0].extra_attributes[0].value, "TXT");
    assert!(
        response
            .write_blockers()
            .contains(&NamecheapWriteBlocker::UnknownHostAttribute)
    );
}

#[test]
fn malformed_or_over_budget_xml_fails_with_redacted_errors() {
    for xml in [
        "<!DOCTYPE ApiResponse><ApiResponse/>",
        "<ApiResponse xmlns:x='urn:a&amp;b'/>",
        "<ApiResponse><unclosed></ApiResponse>",
    ] {
        let error = FacetSourceDecoder.decode(xml).unwrap_err();
        assert_eq!(error, NamecheapAdapterError::MalformedResponse);
        assert!(!error.to_string().contains(xml));
    }
    let nested = format!("{}{}", "<x>".repeat(65), "</x>".repeat(65));
    assert_eq!(
        FacetSourceDecoder.decode(&nested),
        Err(NamecheapAdapterError::MalformedResponse)
    );
    assert_eq!(
        client("x".repeat(1_048_577)).get_hosts(domain()),
        Err(NamecheapAdapterError::ResponseTooLarge)
    );
}

#[test]
fn clean_read_still_cannot_authorize_or_plan_complete_zone_write() {
    let clean = format!(
        "<ApiResponse xmlns='{NS}' Status='OK'><Errors/><RequestedCommand>namecheap.domains.dns.getHosts</RequestedCommand><CommandResponse Type='namecheap.domains.dns.getHosts'><DomainDNSGetHostsResult Domain='example.test' IsUsingOurDNS='true'><Host HostId='1' Name='www' Type='A' Address='192.0.2.1' TTL='1800'/></DomainDNSGetHostsResult></CommandResponse></ApiResponse>"
    );
    let response = client(clean).get_hosts(domain()).unwrap();
    assert!(!response.write_blockers().is_empty());
    assert!(
        response
            .write_blockers()
            .contains(&NamecheapWriteBlocker::UnverifiedEmailServiceMode)
    );
    assert!(
        response
            .write_blockers()
            .contains(&NamecheapWriteBlocker::BrokerOperationApprovalUnavailable)
    );
    let partial = DnsZoneSnapshot {
        domain: "example.test".into(),
        provider: "namecheap-basicdns".into(),
        complete: false,
        records: Vec::new(),
    };
    assert!(
        RecordPlan::build(
            partial,
            RecordChange::Delete {
                host_id: "1".into()
            }
        )
        .is_err()
    );
}
