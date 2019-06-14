use crate::grpc_protobuf_autogen::trace_context::TraceContext;
use protobuf::well_known_types::UInt64Value;

/// Ledger tracing information.
#[derive(Debug, Eq, PartialEq, Default)]
pub struct DamlTraceContext {
    trace_id_high: u64,
    trace_id: u64,
    span_id: u64,
    parent_span_id: u64,
    sampled: bool,
}

impl DamlTraceContext {
    pub fn new(trace_id_high: u64, trace_id: u64, span_id: u64, parent_span_id: u64, sampled: bool) -> Self {
        Self {
            trace_id_high,
            trace_id,
            span_id,
            parent_span_id,
            sampled,
        }
    }

    pub fn trace_id_high(&self) -> u64 {
        self.trace_id_high
    }

    pub fn trace_id(&self) -> u64 {
        self.trace_id
    }

    pub fn span_id(&self) -> u64 {
        self.span_id
    }

    pub fn parent_span_id(&self) -> u64 {
        self.parent_span_id
    }

    pub fn sampled(&self) -> bool {
        self.sampled
    }
}

impl From<TraceContext> for DamlTraceContext {
    fn from(mut trace_context: TraceContext) -> Self {
        Self::new(
            trace_context.get_trace_id_high(),
            trace_context.get_trace_id(),
            trace_context.get_span_id(),
            (trace_context.take_parent_span_id() as UInt64Value).get_value(),
            trace_context.get_sampled(),
        )
    }
}

impl From<DamlTraceContext> for TraceContext {
    fn from(daml_trace_context: DamlTraceContext) -> Self {
        let mut trace_context = Self::new();
        trace_context.set_trace_id_high(daml_trace_context.trace_id_high);
        trace_context.set_trace_id(daml_trace_context.trace_id);
        trace_context.set_span_id(daml_trace_context.span_id);
        let mut parent_span_id = UInt64Value::new();
        parent_span_id.set_value(daml_trace_context.parent_span_id);
        trace_context.set_parent_span_id(parent_span_id);
        trace_context.set_sampled(daml_trace_context.sampled);
        trace_context
    }
}
