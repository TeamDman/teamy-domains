/// Why a `getHosts` read cannot be safely converted to `setHosts` yet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NamecheapWriteBlocker {
    NonAuthoritative,
    MissingRawResponse,
    UnverifiedDnsServiceMode,
    UnknownResultAttribute,
    UnknownResultElement,
    UnknownHostAttribute,
    UnsupportedRecordType,
    UnverifiedEmailServiceMode,
    BrokerOperationApprovalUnavailable,
}
