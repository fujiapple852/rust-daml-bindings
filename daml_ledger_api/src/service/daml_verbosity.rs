/// The verbosity of responses from the DAML ledger.
///
/// If verbose mode is enabled then values served over the API will contain more information than strictly necessary to
/// interpret the data.  In particular, setting the verbose flag to true triggers the ledger to include labels for
/// record fields.
#[derive(Debug, Eq, PartialEq)]
pub enum DamlVerbosity {
    /// Enable verbose mode.
    Verbose,

    /// Disable verbose mode.
    NotVerbose,
}

impl From<DamlVerbosity> for bool {
    fn from(verbose: DamlVerbosity) -> Self {
        verbose == DamlVerbosity::Verbose
    }
}
