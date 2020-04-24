use crate::grpc_protobuf::com::daml::ledger::api::v1::TraceContext;

/// Ledger tracing information.
#[derive(Debug, Eq, PartialEq, Default)]
pub struct DamlTraceContext {
    pub trace_id_high: u64,
    pub trace_id: u64,
    pub span_id: u64,
    pub parent_span_id: u64,
    pub sampled: bool,
}

impl DamlTraceContext {
    pub const fn new(trace_id_high: u64, trace_id: u64, span_id: u64, parent_span_id: u64, sampled: bool) -> Self {
        Self {
            trace_id_high,
            trace_id,
            span_id,
            parent_span_id,
            sampled,
        }
    }

    pub const fn trace_id_high(&self) -> u64 {
        self.trace_id_high
    }

    pub const fn trace_id(&self) -> u64 {
        self.trace_id
    }

    pub const fn span_id(&self) -> u64 {
        self.span_id
    }

    pub const fn parent_span_id(&self) -> u64 {
        self.parent_span_id
    }

    pub const fn sampled(&self) -> bool {
        self.sampled
    }
}

impl From<TraceContext> for DamlTraceContext {
    fn from(trace_context: TraceContext) -> Self {
        Self::new(
            trace_context.trace_id_high,
            trace_context.trace_id,
            trace_context.span_id,
            trace_context.parent_span_id.unwrap_or_default(),
            trace_context.sampled,
        )
    }
}

impl From<DamlTraceContext> for TraceContext {
    fn from(daml_trace_context: DamlTraceContext) -> Self {
        TraceContext {
            trace_id_high: daml_trace_context.trace_id_high,
            trace_id: daml_trace_context.trace_id,
            span_id: daml_trace_context.span_id,
            parent_span_id: Some(daml_trace_context.parent_span_id),
            sampled: daml_trace_context.sampled,
        }
    }
}
