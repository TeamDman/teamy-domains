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
        let mut response = NamecheapHostResponse::from_tree(tree, domain)?;
        response.attach_raw_response(xml);
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::namecheap::{NamecheapRequestFields, NamecheapXmlNode};

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
