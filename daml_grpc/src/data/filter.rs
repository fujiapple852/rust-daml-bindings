use crate::data::identifier::DamlIdentifier;
use crate::grpc_protobuf::com::daml::ledger::api::v1::{Filters, InclusiveFilters, TransactionFilter};
use std::collections::hash_map::HashMap;
use std::ops::Not;

#[derive(Debug, Eq, PartialEq, Clone, Default)]
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
        Filters {
            inclusive: daml_filters.template_ids.is_empty().not().then(|| InclusiveFilters {
                template_ids: daml_filters.template_ids.into_iter().map(Into::into).collect(),
            }),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Default)]
pub struct DamlTransactionFilter {
    filters_by_party: HashMap<String, DamlFilters>,
}

impl DamlTransactionFilter {
    pub const fn filters_by_party(&self) -> &HashMap<String, DamlFilters> {
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
        TransactionFilter {
            filters_by_party: daml_transaction_filter
                .filters_by_party
                .into_iter()
                .map(|(k, v)| (k, Filters::from(v)))
                .collect(),
        }
    }
}
