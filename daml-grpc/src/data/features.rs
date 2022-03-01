use crate::grpc_protobuf::com::daml::ledger::api::v1::{FeaturesDescriptor, UserManagementFeature};

/// The features supported by a Ledger API endpoint.
///
/// Note that `experimental` are excluded here as they are, by definition, unstable.
#[derive(Debug, Eq, PartialEq, Clone, Default)]
pub struct DamlFeaturesDescriptor {
    user_management: Option<DamlUserManagementFeature>,
}

impl DamlFeaturesDescriptor {
    pub fn new(user_management: impl Into<Option<DamlUserManagementFeature>>) -> Self {
        Self {
            user_management: user_management.into(),
        }
    }

    pub fn user_management(&self) -> &Option<DamlUserManagementFeature> {
        &self.user_management
    }
}

impl From<FeaturesDescriptor> for DamlFeaturesDescriptor {
    fn from(features: FeaturesDescriptor) -> Self {
        DamlFeaturesDescriptor::new(features.user_management.map(DamlUserManagementFeature::from))
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Default)]
pub struct DamlUserManagementFeature {
    supported: bool,
}

impl DamlUserManagementFeature {
    pub fn new(supported: bool) -> Self {
        Self {
            supported,
        }
    }

    pub fn supported(&self) -> bool {
        self.supported
    }
}

impl From<UserManagementFeature> for DamlUserManagementFeature {
    fn from(feature: UserManagementFeature) -> Self {
        DamlUserManagementFeature::new(feature.supported)
    }
}
