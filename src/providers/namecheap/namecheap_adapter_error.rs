/// Errors at the provider adapter boundary. Messages never include credentials or raw XML.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NamecheapAdapterError {
    InvalidDomain,
    InvalidRequest,
    MalformedResponse,
    ResponseTooLarge,
    UnexpectedCommand,
    ProviderRejected { code: Option<u32> },
    TransportUnavailable,
    WriteNotQualified,
}

impl std::fmt::Display for NamecheapAdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDomain => f.write_str("invalid Namecheap SLD/TLD"),
            Self::InvalidRequest => f.write_str("invalid Namecheap request"),
            Self::MalformedResponse => f.write_str("malformed Namecheap response"),
            Self::ResponseTooLarge => f.write_str("Namecheap response exceeds offline size limit"),
            Self::UnexpectedCommand => f.write_str("unexpected Namecheap response command"),
            Self::ProviderRejected { code: Some(code) } => {
                write!(f, "Namecheap rejected request with code {code}")
            }
            Self::ProviderRejected { code: None } => f.write_str("Namecheap rejected request"),
            Self::TransportUnavailable => f.write_str("Namecheap transport unavailable"),
            Self::WriteNotQualified => {
                f.write_str("Namecheap write cannot be represented losslessly")
            }
        }
    }
}

impl std::error::Error for NamecheapAdapterError {}
