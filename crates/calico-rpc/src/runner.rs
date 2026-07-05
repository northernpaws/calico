//! The RPC crate defines data structures used
//! to encode/decode metadata about test cases.

/// Represents a test case.
///
/// This is typically decoded from an ELF file by the test
/// runner, and exposed either over the CLI or a machine API.
#[derive(Debug, Clone, PartialEq, postcard_schema::Schema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Test {
    /// The string ID of the test used to identify it.
    pub name: String,
    ///
    pub expected_outcome: TestOutcome,
    /// Indicates if the test should be ignored.
    pub ignored: bool,
    /// Optional timeout of the test run in seconds.
    pub timeout: Option<u32>,
    /// Optional address of the test entrypoint in the ELF.
    ///
    /// Populated if the test has been extracted from an ELF
    /// containing address information of the entrypoint.
    pub address: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, postcard_schema::Schema)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TestOutcome {}
