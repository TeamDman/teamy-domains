/// Decoded XML attribute retained even when the adapter does not understand it.
#[derive(Clone, PartialEq, Eq)]
pub struct NamecheapXmlAttribute {
    /// Namespace URI; unprefixed XML attributes have no namespace.
    pub namespace: Option<String>,
    pub name: String,
    pub value: String,
}

impl std::fmt::Debug for NamecheapXmlAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("NamecheapXmlAttribute([redacted])")
    }
}
