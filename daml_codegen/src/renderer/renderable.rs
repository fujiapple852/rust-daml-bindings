use crate::renderer::{RenderContext, RenderFilterMode};
use daml_lf::element::{DamlArchive, DamlData, DamlKind, DamlTyCon, DamlType};

/// Determine if a type is supported by the renderer.
///
/// When the `filter_mode` is `RenderFilterMode::HKT` then the renderer will exclude fields which contain
/// (recursively) any `TyCon` where the target `DamlData` defines a type parameter with a higher kind (i.e. `* -> *`)
///
/// Note that each `DamlData` item provides kind information (i.e. `DamlData::type_params`) to perform this
/// exclusion logic.  In theory this information could instead be used to generate code which support such higher kinded
/// structures however this is not currently possible in stable Rust to the best of my knowledge.
///
/// When the `filter_mode` is `RenderFilterMode::NonSerializable` then the renderer will exclude all fields
/// which contain (recursively) any `TyCon` where the target `DamlData` has been determined by the DAML compiler to be
/// non-serializable.
///
/// Fields which contain (recursively) types `Arrow`, `Update`, `Scenario`, `Any`, `TypeRep`, `Forall`, `Struct`, `Syn`
/// are always excluded regardless of the mode.  Note that these types are not required for rendering data types that
/// will be used by the DAML Ledger API.
pub struct IsRenderable<'a> {
    archive: &'a DamlArchive<'a>,
    filter_mode: RenderFilterMode,
}

impl<'a> IsRenderable<'a> {
    pub const fn new(ctx: &'a RenderContext<'a>) -> Self {
        Self {
            archive: ctx.archive(),
            filter_mode: ctx.filter_mode(),
        }
    }

    /// Returns `true` if the supplied type can be rendered, `false` otherwise.
    pub fn check_type(&self, ty: &DamlType<'_>) -> bool {
        match ty {
            DamlType::Int64
            | DamlType::Text
            | DamlType::Timestamp
            | DamlType::Party
            | DamlType::Bool
            | DamlType::Unit
            | DamlType::Date
            | DamlType::Nat(_) => true,
            DamlType::Numeric(num) => self.check_type(num.as_ref()),
            DamlType::List(args) | DamlType::TextMap(args) | DamlType::GenMap(args) | DamlType::Optional(args) =>
                args.iter().all(|arg| self.check_type(arg)),
            DamlType::ContractId(tycon) => tycon.as_ref().map_or(true, |ty| self.check_type(ty)),
            DamlType::TyCon(tycon) | DamlType::BoxedTyCon(tycon) => self.check_tycon(tycon),
            DamlType::Var(var) => var.type_arguments().iter().all(|ty| self.check_type(ty)),
            DamlType::Arrow
            | DamlType::Update
            | DamlType::Scenario
            | DamlType::Any
            | DamlType::TypeRep

            // TODO revisit when these types are stable in DAML LF 1.x
            | DamlType::AnyException
            | DamlType::GeneralError
            | DamlType::ArithmeticError
            | DamlType::ContractError
            | DamlType::Bignumeric
            | DamlType::RoundingMode

            | DamlType::Forall(_)
            | DamlType::Struct(_)
            | DamlType::Syn(_) => false,
        }
    }

    fn check_tycon(&self, tycon: &DamlTyCon<'_>) -> bool {
        self.check_target_data(tycon) && self.check_tycon_type_arguments(tycon)
    }

    fn check_target_data(&self, tycon: &DamlTyCon<'_>) -> bool {
        match self.filter_mode {
            RenderFilterMode::HigherKindedType => self.archive.data_by_tycon(tycon).map_or(true, |data| {
                !data.type_params().iter().any(|type_var| matches!(type_var.kind(), DamlKind::Arrow(_)))
            }),
            RenderFilterMode::NonSerializable => self.archive.data_by_tycon(tycon).map_or(true, DamlData::serializable),
        }
    }

    fn check_tycon_type_arguments(&self, tycon: &DamlTyCon<'_>) -> bool {
        tycon.type_arguments().iter().all(|ty| self.check_type(ty))
    }
}
