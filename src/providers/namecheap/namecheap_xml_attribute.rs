/// Decoded XML attribute retained even when the adapter does not understand it.
#[derive(Clone, PartialEq, Eq)]
pub struct NamecheapXmlAttribute {
    pub name: String,
    pub value: String,
}

impl std::fmt::Debug for NamecheapXmlAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("NamecheapXmlAttribute([redacted])")
    }
}
