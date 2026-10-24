//! Exact-source qualification adapter. Not linked into the production CLI.

use facet_xml::StrictXmlLimits;
use facet_xml_node::{StrictContent, StrictDocument, StrictElement};
use teamy_domains::providers::namecheap::{
    NamecheapAdapterError, NamecheapXmlAttribute, NamecheapXmlDecoder, NamecheapXmlNode,
};

/// Decode synthetic Namecheap responses through the owned Facet strict parser.
pub struct FacetSourceDecoder;

impl NamecheapXmlDecoder for FacetSourceDecoder {
    fn decode(&self, xml: &str) -> Result<NamecheapXmlNode, NamecheapAdapterError> {
        let document = StrictDocument::parse(xml.as_bytes(), StrictXmlLimits::default())
            .map_err(|_| NamecheapAdapterError::MalformedResponse)?;
        convert_element(&document, document.root())
    }
}

fn convert_element(
    document: &StrictDocument,
    element: &StrictElement,
) -> Result<NamecheapXmlNode, NamecheapAdapterError> {
    let raw_xml = document
        .raw_node(element)
        .and_then(|raw| std::str::from_utf8(raw).ok())
        .ok_or(NamecheapAdapterError::MalformedResponse)?
        .to_owned();
    let mut children = Vec::new();
    let mut text = String::new();
    for child in element.children() {
        match child {
            StrictContent::Text(value) => text.push_str(value),
            StrictContent::Element(child) => children.push(convert_element(document, child)?),
        }
    }
    Ok(NamecheapXmlNode {
        namespace: element.name().namespace().map(str::to_owned),
        name: element.name().local().to_owned(),
        attributes: element
            .attributes()
            .iter()
            .map(|attribute| NamecheapXmlAttribute {
                namespace: attribute.name().namespace().map(str::to_owned),
                name: attribute.name().local().to_owned(),
                value: attribute.value().to_owned(),
            })
            .collect(),
        text,
        children,
        raw_xml: Some(raw_xml),
    })
}
