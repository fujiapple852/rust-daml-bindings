use crate::data::identifier::DamlIdentifier;
use crate::grpc_protobuf_autogen::transaction_filter::Filters;
use crate::grpc_protobuf_autogen::transaction_filter::InclusiveFilters;
use crate::grpc_protobuf_autogen::transaction_filter::TransactionFilter;
use std::collections::hash_map::HashMap;

#[derive(Debug, Eq, PartialEq, Default)]
pub struct DamlFilters {
    template_ids: Vec<DamlIdentifier>,
}

impl DamlFilters {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn template_ids(&self) -> &[DamlIdentifier] {
        &self.template_ids
    }
}

impl From<DamlFilters> for Filters {
    fn from(daml_filters: DamlFilters) -> Self {
        let mut filters = Self::new();
        if !daml_filters.template_ids.is_empty() {
            let mut inclusive = InclusiveFilters::new();
            inclusive.set_template_ids(daml_filters.template_ids.into_iter().map(Into::into).collect());
            filters.set_inclusive(inclusive);
        }
        filters
    }
}

#[derive(Debug, Eq, PartialEq, Default)]
pub struct DamlTransactionFilter {
    filters_by_party: HashMap<String, DamlFilters>,
}

impl DamlTransactionFilter {
    pub fn filters_by_party(&self) -> &HashMap<String, DamlFilters> {
        &self.filters_by_party
    }
}

impl DamlTransactionFilter {
    pub fn for_parties<P, S>(parties: P) -> Self
    where
        P: Into<Vec<S>>,
        S: Into<String>,
    {
        Self {
            filters_by_party: parties.into().into_iter().map(|p| (p.into(), DamlFilters::new())).collect(),
        }
    }
}

impl From<DamlTransactionFilter> for TransactionFilter {
    fn from(daml_transaction_filter: DamlTransactionFilter) -> Self {
        let mut filter = Self::new();
        filter.set_filters_by_party(
            daml_transaction_filter.filters_by_party.into_iter().map(|(k, v)| (k, v.into())).collect(),
        );
        filter
    }
}
