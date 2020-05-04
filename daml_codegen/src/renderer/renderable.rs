use crate::renderer::RenderContext;
use daml_lf::element::{DamlArchive, DamlDataRef, DamlKind, DamlType};

/// Determine if a type is supported by the code generator.
///
/// The Rust code renderer does not support the following types:
///
/// - `Arrow`, `Update`, `Scenario`, `Any`, `TypeRep`
/// - `DataRef` if the target `DamlData` defines a type parameter with a higher kind (i.e. `* -> *`)
/// - Any types which contains any nested type parameter (recursively) of an unsupported type
///
/// The code generator will omit the entire `DamlField` if the `DamlType` of that field cannot be rendered.
///
/// Note that each `DamlData` item provides kind information (i.e. `DamlData::type_arguments`) to perform this
/// exclusion logic.  In theory this information could instead be used to generate code which support such higher kinded
/// structures however this is not currently possible in stable Rust to the best of my knowledge.
pub struct IsRenderable<'a> {
    archive: &'a DamlArchive<'a>,
}

impl<'a> IsRenderable<'a> {
    pub fn new(ctx: &'a RenderContext<'a>) -> Self {
        Self {
            archive: ctx.archive(),
        }
    }

    /// Returns `true` if the supplied type can be rendered, `false` otherwise.
    pub fn check_type(&self, ty: &DamlType<'_>) -> bool {
        match ty {
            DamlType::Int64
            | DamlType::Numeric(_)
            | DamlType::Text
            | DamlType::Timestamp
            | DamlType::Party
            | DamlType::Bool
            | DamlType::Unit
            | DamlType::Date
            | DamlType::Nat(_) => true,
            DamlType::List(inner) | DamlType::TextMap(inner) | DamlType::Optional(inner) => self.check_type(inner),
            DamlType::ContractId(data_ref) => data_ref.as_ref().map_or(true, |dr| self.check_data_ref(dr)),
            DamlType::DataRef(data_ref) | DamlType::BoxedDataRef(data_ref) => self.check_data_ref(data_ref),
            DamlType::Var(var) => var.type_arguments().iter().all(|ty| self.check_type(ty)),
            DamlType::Arrow | DamlType::Update | DamlType::Scenario | DamlType::Any | DamlType::TypeRep => false,
        }
    }

    fn check_data_ref(&self, data_ref: &DamlDataRef<'_>) -> bool {
        self.check_target_data(data_ref) && self.check_data_ref_type_arguments(data_ref)
    }

    fn check_target_data(&self, data_ref: &DamlDataRef<'_>) -> bool {
        self.archive.data_by_ref(data_ref).map_or(true, |data| {
            !data.type_arguments().iter().any(|type_var| matches!(type_var.kind(), DamlKind::Arrow(_)))
        })
    }

    fn check_data_ref_type_arguments(&self, data_ref: &DamlDataRef<'_>) -> bool {
        data_ref.type_arguments().iter().all(|ty| self.check_type(ty))
    }
}
