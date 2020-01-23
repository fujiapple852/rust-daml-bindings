use crate::data::identifier::DamlIdentifier;
use crate::grpc_protobuf::com::digitalasset::ledger::api::v1::{Filters, InclusiveFilters, TransactionFilter};
use std::collections::hash_map::HashMap;

#[derive(Debug, Eq, PartialEq, Default)]
pub struct DamlFilters {
    pub template_ids: Vec<DamlIdentifier>,
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
            inclusive: if daml_filters.template_ids.is_empty() {
                None
            } else {
                Some(InclusiveFilters {
                    template_ids: daml_filters.template_ids.into_iter().map(Into::into).collect(),
                })
            },
        }
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
        TransactionFilter {
            filters_by_party: daml_transaction_filter
                .filters_by_party
                .into_iter()
                .map(|(k, v)| (k, Filters::from(v)))
                .collect(),
        }
    }
}
