use super::NamecheapXmlAttribute;

/// Parsed XML tree retaining unknown elements and attributes. A real decoder must enforce depth and node limits.
#[derive(Clone, PartialEq, Eq)]
pub struct NamecheapXmlNode {
    pub name: String,
    pub attributes: Vec<NamecheapXmlAttribute>,
    pub text: String,
    pub children: Vec<Self>,
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
            .find(|a| a.name == name)
            .map(|a| a.value.as_str())
    }
}
