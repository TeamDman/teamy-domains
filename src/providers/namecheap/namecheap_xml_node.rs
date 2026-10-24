use super::NamecheapXmlAttribute;

/// Parsed XML tree retaining unknown expanded names and the exact lexical source.
#[derive(Clone, PartialEq, Eq)]
pub struct NamecheapXmlNode {
    /// Namespace URI, including one inherited from a default declaration.
    pub namespace: Option<String>,
    pub name: String,
    pub attributes: Vec<NamecheapXmlAttribute>,
    pub text: String,
    pub children: Vec<Self>,
    /// Exact source slice for this element; children may inherit declarations outside it.
    pub raw_xml: Option<String>,
}

impl std::fmt::Debug for NamecheapXmlNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("NamecheapXmlNode([redacted])")
    }
}

impl NamecheapXmlNode {
    #[must_use]
    pub fn attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|a| a.namespace.is_none() && a.name == name)
            .map(|a| a.value.as_str())
    }
}
