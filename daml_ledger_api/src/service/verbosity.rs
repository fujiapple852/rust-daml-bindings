/// The verbosity of responses from the DAML ledger.
///
/// If verbose mode is enabled then values served over the API will contain more information than strictly necessary to
/// interpret the data.  In particular, setting the verbose flag to true triggers the ledger to include labels for
/// record fields.
pub enum DamlVerbosity {
    /// Do not specify a verbosity mode (use default ledger behaviour).
    LedgerDefault,

    /// Enable verbose mode.
    Verbose,

    /// Disable verbose mode.
    NotVerbose,
}
