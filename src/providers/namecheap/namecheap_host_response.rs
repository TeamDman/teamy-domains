use super::{
    NamecheapAdapterError, NamecheapDomainId, NamecheapHostId, NamecheapHostRow,
    NamecheapWriteBlocker, NamecheapXmlAttribute, NamecheapXmlNode,
};

/// A successful `getHosts` result. Raw extras remain visible to a future adapter.
#[derive(Clone, PartialEq, Eq)]
pub struct NamecheapHostResponse {
    domain: NamecheapDomainId,
    using_namecheap_dns: bool,
    hosts: Vec<NamecheapHostRow>,
    extra_attributes: Vec<NamecheapXmlAttribute>,
    extra_elements: Vec<NamecheapXmlNode>,
    raw_tree: NamecheapXmlNode,
    unrecognized_envelope: bool,
}

impl std::fmt::Debug for NamecheapHostResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("NamecheapHostResponse([redacted])")
    }
}

impl NamecheapHostResponse {
    /// Decode a typed `getHosts` result from a retained XML tree.
    /// # Errors
    /// Rejects malformed, rejected, mismatched or ambiguous provider responses.
    pub(crate) fn from_tree(
        root: NamecheapXmlNode,
        expected_domain: NamecheapDomainId,
    ) -> Result<Self, NamecheapAdapterError> {
        if root.name != "ApiResponse" {
            return Err(NamecheapAdapterError::MalformedResponse);
        }
        if required_attribute(&root, "xmlns")? != "http://api.namecheap.com/xml.response" {
            return Err(NamecheapAdapterError::MalformedResponse);
        }
        match required_attribute(&root, "Status")? {
            "ERROR" => {
                let code = required_child(&root, "Errors")
                    .ok()
                    .and_then(|errors| required_child(errors, "Error").ok())
                    .and_then(|error| error.attribute("Number"))
                    .and_then(|number| number.parse().ok());
                return Err(NamecheapAdapterError::ProviderRejected { code });
            }
            "OK" => {}
            _ => return Err(NamecheapAdapterError::MalformedResponse),
        }
        if !required_child(&root, "Errors")?.children.is_empty() {
            return Err(NamecheapAdapterError::MalformedResponse);
        }
        let errors = required_child(&root, "Errors")?;
        if !errors.attributes.is_empty() || !errors.text.trim().is_empty() {
            return Err(NamecheapAdapterError::MalformedResponse);
        }
        let method = "namecheap.domains.dns.getHosts";
        if required_child(&root, "RequestedCommand")?.text.trim() != method {
            return Err(NamecheapAdapterError::UnexpectedCommand);
        }
        let command = required_child(&root, "CommandResponse")?;
        if required_attribute(command, "Type")? != method {
            return Err(NamecheapAdapterError::UnexpectedCommand);
        }
        let result = required_child(command, "DomainDNSGetHostsResult")?;
        if required_attribute(result, "Domain")? != expected_domain.full_name() {
            return Err(NamecheapAdapterError::MalformedResponse);
        }
        let using_namecheap_dns = match required_attribute(result, "IsUsingOurDNS")? {
            "true" | "True" => true,
            "false" | "False" => false,
            _ => return Err(NamecheapAdapterError::MalformedResponse),
        };
        let (hosts, extra_elements, unknown_host_content) = parse_hosts(result)?;
        let unrecognized_envelope =
            has_unrecognized_envelope(&root, command, result) || unknown_host_content;
        let extra_attributes = result
            .attributes
            .iter()
            .filter(|a| !matches!(a.name.as_str(), "Domain" | "IsUsingOurDNS"))
            .cloned()
            .collect();
        Ok(Self {
            domain: expected_domain,
            using_namecheap_dns,
            hosts,
            extra_attributes,
            extra_elements,
            raw_tree: root,
            unrecognized_envelope,
        })
    }

    /// Typed reasons that keep whole-zone writes disabled, including gaps a clean sample cannot rule out.
    #[must_use]
    pub fn write_blockers(&self) -> Vec<NamecheapWriteBlocker> {
        let mut blockers = Vec::new();
        if !self.using_namecheap_dns {
            blockers.push(NamecheapWriteBlocker::NonAuthoritative);
        }
        // IsUsingOurDNS can include services that getHosts/setHosts cannot manage.
        blockers.push(NamecheapWriteBlocker::UnverifiedDnsServiceMode);
        if !self.extra_attributes.is_empty() {
            blockers.push(NamecheapWriteBlocker::UnknownResultAttribute);
        }
        if !self.extra_elements.is_empty() || self.unrecognized_envelope {
            blockers.push(NamecheapWriteBlocker::UnknownResultElement);
        }
        if self.hosts.iter().any(|h| !h.extra_attributes.is_empty()) {
            blockers.push(NamecheapWriteBlocker::UnknownHostAttribute);
        }
        if self.hosts.iter().any(|h| {
            !matches!(
                h.record_type.as_str(),
                "A" | "AAAA"
                    | "ALIAS"
                    | "CAA"
                    | "CNAME"
                    | "MX"
                    | "MXE"
                    | "NS"
                    | "TXT"
                    | "URL"
                    | "URL301"
                    | "FRAME"
            )
        }) {
            blockers.push(NamecheapWriteBlocker::UnsupportedRecordType);
        }
        // getHosts does not establish EmailType or all account service settings for setHosts.
        blockers.push(NamecheapWriteBlocker::UnverifiedEmailServiceMode);
        blockers.push(NamecheapWriteBlocker::BrokerOperationApprovalUnavailable);
        blockers
    }

    #[must_use]
    pub fn domain(&self) -> &NamecheapDomainId {
        &self.domain
    }

    #[must_use]
    pub fn hosts(&self) -> &[NamecheapHostRow] {
        &self.hosts
    }

    #[must_use]
    pub fn extra_elements(&self) -> &[NamecheapXmlNode] {
        &self.extra_elements
    }

    #[must_use]
    pub fn raw_tree(&self) -> &NamecheapXmlNode {
        &self.raw_tree
    }
}

fn has_unrecognized_envelope(
    root: &NamecheapXmlNode,
    command: &NamecheapXmlNode,
    result: &NamecheapXmlNode,
) -> bool {
    !root.text.trim().is_empty()
        || root
            .attributes
            .iter()
            .any(|a| !matches!(a.name.as_str(), "Status" | "xmlns"))
        || root.children.iter().any(|c| {
            !matches!(
                c.name.as_str(),
                "Errors"
                    | "RequestedCommand"
                    | "CommandResponse"
                    | "Server"
                    | "GMTTimeDifference"
                    | "ExecutionTime"
            )
        })
        || root
            .children
            .iter()
            .filter(|c| {
                matches!(
                    c.name.as_str(),
                    "RequestedCommand" | "Server" | "GMTTimeDifference" | "ExecutionTime"
                )
            })
            .any(|c| !c.attributes.is_empty() || !c.children.is_empty())
        || command.attributes.iter().any(|a| a.name != "Type")
        || !command.text.trim().is_empty()
        || command
            .children
            .iter()
            .any(|c| c.name != "DomainDNSGetHostsResult")
        || !result.text.trim().is_empty()
}

fn parse_hosts(
    result: &NamecheapXmlNode,
) -> Result<(Vec<NamecheapHostRow>, Vec<NamecheapXmlNode>, bool), NamecheapAdapterError> {
    let mut hosts = Vec::new();
    let mut host_ids = std::collections::HashSet::new();
    let mut extra_elements = Vec::new();
    let mut unknown_host_content = false;
    for child in &result.children {
        if child.name == "Host" {
            if !child.children.is_empty() || !child.text.trim().is_empty() {
                unknown_host_content = true;
            }
            let host_id = NamecheapHostId::new(required_attribute(child, "HostId")?)?;
            if !host_ids.insert(host_id.as_str().to_owned()) {
                return Err(NamecheapAdapterError::MalformedResponse);
            }
            hosts.push(NamecheapHostRow {
                host_id,
                name: required_attribute(child, "Name")?.into(),
                record_type: required_attribute(child, "Type")?.into(),
                address: required_attribute(child, "Address")?.into(),
                mx_pref: optional_attribute(child, "MXPref")?.map(str::to_owned),
                ttl: optional_attribute(child, "TTL")?.map(str::to_owned),
                extra_attributes: child
                    .attributes
                    .iter()
                    .filter(|a| {
                        !matches!(
                            a.name.as_str(),
                            "HostId" | "Name" | "Type" | "Address" | "MXPref" | "TTL"
                        )
                    })
                    .cloned()
                    .collect(),
            });
        } else {
            extra_elements.push(child.clone());
        }
    }
    Ok((hosts, extra_elements, unknown_host_content))
}

fn optional_attribute<'a>(
    node: &'a NamecheapXmlNode,
    name: &str,
) -> Result<Option<&'a str>, NamecheapAdapterError> {
    let mut matches = node.attributes.iter().filter(|a| a.name == name);
    let first = matches.next().map(|a| a.value.as_str());
    if matches.next().is_some() {
        return Err(NamecheapAdapterError::MalformedResponse);
    }
    Ok(first)
}

fn required_attribute<'a>(
    node: &'a NamecheapXmlNode,
    name: &str,
) -> Result<&'a str, NamecheapAdapterError> {
    optional_attribute(node, name)?.ok_or(NamecheapAdapterError::MalformedResponse)
}

fn required_child<'a>(
    node: &'a NamecheapXmlNode,
    name: &str,
) -> Result<&'a NamecheapXmlNode, NamecheapAdapterError> {
    let mut matches = node.children.iter().filter(|c| c.name == name);
    let first = matches
        .next()
        .ok_or(NamecheapAdapterError::MalformedResponse)?;
    if matches.next().is_some() {
        return Err(NamecheapAdapterError::MalformedResponse);
    }
    Ok(first)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(
        name: &str,
        attributes: &[(&str, &str)],
        text: &str,
        children: Vec<NamecheapXmlNode>,
    ) -> NamecheapXmlNode {
        NamecheapXmlNode {
            name: name.into(),
            attributes: attributes
                .iter()
                .map(|(name, value)| NamecheapXmlAttribute {
                    name: (*name).into(),
                    value: (*value).into(),
                })
                .collect(),
            text: text.into(),
            children,
        }
    }

    fn tree_with_host(host: NamecheapXmlNode) -> NamecheapXmlNode {
        node(
            "ApiResponse",
            &[
                ("Status", "OK"),
                ("xmlns", "http://api.namecheap.com/xml.response"),
            ],
            "",
            vec![
                node("Errors", &[], "", Vec::new()),
                node(
                    "RequestedCommand",
                    &[],
                    "namecheap.domains.dns.getHosts",
                    Vec::new(),
                ),
                node(
                    "CommandResponse",
                    &[("Type", "namecheap.domains.dns.getHosts")],
                    "",
                    vec![node(
                        "DomainDNSGetHostsResult",
                        &[("Domain", "example.test"), ("IsUsingOurDNS", "true")],
                        "",
                        vec![host],
                    )],
                ),
            ],
        )
    }

    fn sample() -> NamecheapHostResponse {
        NamecheapHostResponse {
            domain: NamecheapDomainId::new("example", "test").unwrap(),
            using_namecheap_dns: true,
            hosts: vec![NamecheapHostRow {
                host_id: NamecheapHostId::new("12").unwrap(),
                name: "www".into(),
                record_type: "A".into(),
                address: "192.0.2.4".into(),
                mx_pref: Some("10".into()),
                ttl: Some("1800".into()),
                extra_attributes: Vec::new(),
            }],
            extra_attributes: Vec::new(),
            extra_elements: Vec::new(),
            raw_tree: NamecheapXmlNode {
                name: "ApiResponse".into(),
                attributes: Vec::new(),
                text: String::new(),
                children: Vec::new(),
            },
            unrecognized_envelope: false,
        }
    }

    #[test]
    fn even_a_clean_sample_cannot_authorize_a_whole_zone_write() {
        let blockers = sample().write_blockers();
        assert!(blockers.contains(&NamecheapWriteBlocker::UnverifiedEmailServiceMode));
        assert!(blockers.contains(&NamecheapWriteBlocker::BrokerOperationApprovalUnavailable));
    }

    #[test]
    fn unknown_fields_and_records_are_retained_and_block_writes() {
        let mut response = sample();
        response.extra_attributes.push(NamecheapXmlAttribute {
            name: "FutureService".into(),
            value: "keep".into(),
        });
        response.hosts[0]
            .extra_attributes
            .push(NamecheapXmlAttribute {
                name: "FutureFlag".into(),
                value: "keep".into(),
            });
        response.hosts[0].record_type = "FUTURE".into();
        response.extra_elements.push(NamecheapXmlNode {
            name: "FutureChild".into(),
            attributes: Vec::new(),
            text: "preserve me".into(),
            children: Vec::new(),
        });
        response.using_namecheap_dns = false;
        let blockers = response.write_blockers();
        for blocker in [
            NamecheapWriteBlocker::NonAuthoritative,
            NamecheapWriteBlocker::UnknownResultAttribute,
            NamecheapWriteBlocker::UnknownResultElement,
            NamecheapWriteBlocker::UnknownHostAttribute,
            NamecheapWriteBlocker::UnsupportedRecordType,
        ] {
            assert!(blockers.contains(&blocker));
        }
        assert_eq!(response.extra_attributes[0].value, "keep");
        assert_eq!(response.hosts[0].extra_attributes[0].value, "keep");
        assert_eq!(response.extra_elements[0].text, "preserve me");
    }

    #[test]
    fn tree_decoder_checks_command_domain_and_retains_unknown_host_attribute() {
        let host = node(
            "Host",
            &[
                ("HostId", "12"),
                ("Name", "www"),
                ("Type", "A"),
                ("Address", "192.0.2.4"),
                ("TTL", "1800"),
                ("FutureFlag", "keep"),
            ],
            "",
            Vec::new(),
        );
        let tree = tree_with_host(host);
        let domain = NamecheapDomainId::new("example", "test").unwrap();
        let result = NamecheapHostResponse::from_tree(tree.clone(), domain.clone()).unwrap();
        assert_eq!(result.raw_tree, tree);
        assert_eq!(result.hosts[0].host_id.as_str(), "12");
        assert_eq!(result.hosts[0].extra_attributes[0].value, "keep");
        assert!(
            result
                .write_blockers()
                .contains(&NamecheapWriteBlocker::UnknownHostAttribute)
        );
        let other = NamecheapDomainId::new("different", "test").unwrap();
        assert_eq!(
            NamecheapHostResponse::from_tree(tree, other),
            Err(NamecheapAdapterError::MalformedResponse)
        );
    }

    #[test]
    fn duplicate_ids_and_wrong_namespace_are_rejected() {
        let host = node(
            "Host",
            &[
                ("HostId", "12"),
                ("Name", "www"),
                ("Type", "A"),
                ("Address", "192.0.2.4"),
            ],
            "",
            Vec::new(),
        );
        let mut tree = tree_with_host(host.clone());
        tree.children[2].children[0].children.push(host);
        let domain = NamecheapDomainId::new("example", "test").unwrap();
        assert_eq!(
            NamecheapHostResponse::from_tree(tree, domain.clone()),
            Err(NamecheapAdapterError::MalformedResponse)
        );

        let mut tree = tree_with_host(node(
            "Host",
            &[
                ("HostId", "12"),
                ("Name", "www"),
                ("Type", "A"),
                ("Address", "192.0.2.4"),
            ],
            "",
            Vec::new(),
        ));
        tree.attributes[1].value = "https://unrelated.example.test/xml".into();
        assert_eq!(
            NamecheapHostResponse::from_tree(tree, domain),
            Err(NamecheapAdapterError::MalformedResponse)
        );
    }

    #[test]
    fn unexpected_envelope_content_is_retained_and_blocks_writes() {
        let host = node(
            "Host",
            &[
                ("HostId", "12"),
                ("Name", "www"),
                ("Type", "A"),
                ("Address", "192.0.2.4"),
            ],
            "",
            Vec::new(),
        );
        let mut tree = tree_with_host(host);
        tree.attributes.push(NamecheapXmlAttribute {
            name: "FutureEnvelope".into(),
            value: "keep".into(),
        });
        tree.children[1].attributes.push(NamecheapXmlAttribute {
            name: "FutureCommandField".into(),
            value: "keep".into(),
        });
        let response = NamecheapHostResponse::from_tree(
            tree.clone(),
            NamecheapDomainId::new("example", "test").unwrap(),
        )
        .unwrap();
        assert_eq!(response.raw_tree, tree);
        assert!(
            response
                .write_blockers()
                .contains(&NamecheapWriteBlocker::UnknownResultElement)
        );
    }

    #[test]
    fn error_tree_returns_provider_code_without_raw_message() {
        let tree = node(
            "ApiResponse",
            &[
                ("Status", "ERROR"),
                ("xmlns", "http://api.namecheap.com/xml.response"),
            ],
            "",
            vec![node(
                "Errors",
                &[],
                "",
                vec![node(
                    "Error",
                    &[("Number", "2030288")],
                    "private provider text",
                    Vec::new(),
                )],
            )],
        );
        let error = NamecheapHostResponse::from_tree(
            tree,
            NamecheapDomainId::new("example", "test").unwrap(),
        )
        .unwrap_err();
        assert_eq!(
            error,
            NamecheapAdapterError::ProviderRejected {
                code: Some(2_030_288)
            }
        );
        assert!(!error.to_string().contains("private provider text"));
    }
}
