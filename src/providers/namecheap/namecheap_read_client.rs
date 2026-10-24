use super::{
    NamecheapAdapterError, NamecheapDomainId, NamecheapHostResponse, NamecheapRequest,
    NamecheapTransport, NamecheapXmlDecoder,
};

/// Offline provider adapter with injected transport and XML decoder.
pub struct NamecheapReadClient<T, D> {
    transport: T,
    decoder: D,
}

impl<T, D> std::fmt::Debug for NamecheapReadClient<T, D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("NamecheapReadClient([redacted])")
    }
}

impl<T: NamecheapTransport, D: NamecheapXmlDecoder> NamecheapReadClient<T, D> {
    #[must_use]
    pub fn new(transport: T, decoder: D) -> Self {
        Self { transport, decoder }
    }

    /// # Errors
    /// Rejects invalid requests, transport failure or unexpected provider response shape.
    pub fn get_hosts(
        &self,
        domain: NamecheapDomainId,
    ) -> Result<NamecheapHostResponse, NamecheapAdapterError> {
        let fields = NamecheapRequest::HostRecords(domain.clone()).fields()?;
        let xml = self.transport.exchange(&fields)?;
        if xml.len() > 1_048_576 {
            return Err(NamecheapAdapterError::ResponseTooLarge);
        }
        let tree = self.decoder.decode(&xml)?;
        NamecheapHostResponse::from_tree(tree, domain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::namecheap::{
        NamecheapRequestFields, NamecheapWriteBlocker, NamecheapXmlAttribute, NamecheapXmlNode,
    };
    use std::cell::Cell;

    const FIXTURE: &str = include_str!("../../../tests/fixtures/namecheap/get_hosts_unknown.xml");

    struct MockTransport {
        calls: Cell<u32>,
    }
    impl NamecheapTransport for MockTransport {
        fn exchange(
            &self,
            request: &NamecheapRequestFields,
        ) -> Result<String, NamecheapAdapterError> {
            assert_eq!(request.method(), "namecheap.domains.dns.getHosts");
            assert_eq!(
                request.params(),
                &[
                    ("SLD".into(), "example".into()),
                    ("TLD".into(), "test".into())
                ]
            );
            self.calls.set(self.calls.get() + 1);
            Ok(FIXTURE.into())
        }
    }

    // This test decoder stands in for reviewed XML tokenization. It verifies fixture routing,
    // not XML parsing; no production decoder exists yet.
    struct FixtureDecoder;
    impl NamecheapXmlDecoder for FixtureDecoder {
        fn decode(&self, xml: &str) -> Result<NamecheapXmlNode, NamecheapAdapterError> {
            assert_eq!(xml, FIXTURE);
            let attr = |name: &str, value: &str| NamecheapXmlAttribute {
                name: name.into(),
                value: value.into(),
            };
            let node = |name: &str,
                        attributes: Vec<NamecheapXmlAttribute>,
                        text: &str,
                        children: Vec<NamecheapXmlNode>| NamecheapXmlNode {
                name: name.into(),
                attributes,
                text: text.into(),
                children,
            };
            Ok(node(
                "ApiResponse",
                vec![
                    attr("Status", "OK"),
                    attr("xmlns", "http://api.namecheap.com/xml.response"),
                ],
                "",
                vec![
                    node("Errors", Vec::new(), "", Vec::new()),
                    node(
                        "RequestedCommand",
                        Vec::new(),
                        "namecheap.domains.dns.getHosts",
                        Vec::new(),
                    ),
                    node(
                        "CommandResponse",
                        vec![attr("Type", "namecheap.domains.dns.getHosts")],
                        "",
                        vec![node(
                            "DomainDNSGetHostsResult",
                            vec![
                                attr("Domain", "example.test"),
                                attr("IsUsingOurDNS", "true"),
                                attr("FutureService", "keep"),
                            ],
                            "",
                            vec![
                                node(
                                    "Host",
                                    vec![
                                        attr("HostId", "21"),
                                        attr("Name", "_future"),
                                        attr("Type", "FUTURE"),
                                        attr("Address", "opaque"),
                                        attr("TTL", "1800"),
                                        attr("FutureFlag", "keep"),
                                    ],
                                    "",
                                    Vec::new(),
                                ),
                                node(
                                    "FutureChild",
                                    vec![attr("Enabled", "true")],
                                    "preserve me",
                                    Vec::new(),
                                ),
                            ],
                        )],
                    ),
                ],
            ))
        }
    }

    #[test]
    fn mock_transport_passes_fixture_to_typed_decoder_without_credential_fields() {
        let client = NamecheapReadClient::new(
            MockTransport {
                calls: Cell::new(0),
            },
            FixtureDecoder,
        );
        let response = client
            .get_hosts(NamecheapDomainId::new("example", "test").unwrap())
            .unwrap();
        assert_eq!(client.transport.calls.get(), 1);
        assert_eq!(response.hosts()[0].extra_attributes[0].value, "keep");
        assert_eq!(response.extra_elements()[0].text, "preserve me");
        assert!(
            response
                .write_blockers()
                .contains(&NamecheapWriteBlocker::UnknownHostAttribute)
        );
    }

    struct OversizedTransport;
    impl NamecheapTransport for OversizedTransport {
        fn exchange(
            &self,
            _request: &NamecheapRequestFields,
        ) -> Result<String, NamecheapAdapterError> {
            Ok("x".repeat(1_048_577))
        }
    }
    struct DecoderMustNotRun;
    impl NamecheapXmlDecoder for DecoderMustNotRun {
        fn decode(&self, _xml: &str) -> Result<NamecheapXmlNode, NamecheapAdapterError> {
            panic!("oversized XML must be rejected before decoding")
        }
    }

    #[test]
    fn oversized_response_is_rejected_before_decoder() {
        let client = NamecheapReadClient::new(OversizedTransport, DecoderMustNotRun);
        let result = client.get_hosts(NamecheapDomainId::new("example", "test").unwrap());
        assert_eq!(result, Err(NamecheapAdapterError::ResponseTooLarge));
    }
}
