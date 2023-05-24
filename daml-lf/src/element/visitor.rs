#[cfg(feature = "full")]
use crate::element::daml_expr::DamlScenario;
#[cfg(feature = "full")]
use crate::element::{
    DamlAbs, DamlApp, DamlBinding, DamlBlock, DamlBuiltinFunction, DamlCase, DamlCaseAlt, DamlCaseAltCons,
    DamlCaseAltEnum, DamlCaseAltOptionalSome, DamlCaseAltSum, DamlCaseAltVariant, DamlCommit, DamlCons, DamlCreate,
    DamlDefValue, DamlEnumCon, DamlExercise, DamlExerciseByKey, DamlExpr, DamlFetch, DamlFieldWithExpr, DamlFromAny,
    DamlFromAnyException, DamlLocalValueName, DamlNonLocalValueName, DamlOptionalSome, DamlPrimCon, DamlPrimLit,
    DamlPure, DamlRecCon, DamlRecProj, DamlRecUpd, DamlRetrieveByKey, DamlScenarioEmbedExpr, DamlStructCon,
    DamlStructProj, DamlStructUpd, DamlThrow, DamlToAny, DamlToAnyException, DamlTryCatch, DamlTyAbs, DamlTyApp,
    DamlUpdate, DamlUpdateEmbedExpr, DamlValueName, DamlVarWithType, DamlVariantCon, RoundingMode,
};
use crate::element::{
    DamlAbsoluteTyCon, DamlArchive, DamlArrow, DamlChoice, DamlData, DamlDefKey, DamlDefTypeSyn, DamlEnum, DamlField,
    DamlForall, DamlKind, DamlLocalTyCon, DamlModule, DamlNonLocalTyCon, DamlPackage, DamlRecord, DamlStruct, DamlSyn,
    DamlTemplate, DamlTyCon, DamlTyConName, DamlType, DamlTypeVarWithKind, DamlVar, DamlVariant,
};

/// A Daml [element](`crate::element`) that can be visited by a [`DamlElementVisitor`].
///
/// See [`DamlElementVisitor`].
pub trait DamlVisitableElement<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor);
}

/// A Daml element visitor.
///
/// Visit a tree of Daml [element](`crate::element`) types and apply an action.
///
/// # Examples
///
/// The following example opens a Daml [`DarFile`](`crate::DarFile`) and applies a visitor which records the names of
/// all [`DamlEnum`] data items present in the tree.
///
/// ```no_run
/// # use std::collections::HashSet;
/// # use daml_lf::{DarFile, DamlLfResult};
/// # use daml_lf::element::{DamlElementVisitor, DamlVisitableElement, DamlEnum};
/// # fn main() -> DamlLfResult<()> {
/// #[derive(Default)]
/// pub struct GatherEnumsVisitor(HashSet<String>);
///
/// impl DamlElementVisitor for GatherEnumsVisitor {
///     fn pre_visit_enum<'a>(&mut self, data_enum: &'a DamlEnum<'a>) {
///         self.0.insert(data_enum.name().to_owned());
///     }
/// }
///
/// let mut visitor = GatherEnumsVisitor::default();
/// let dar = DarFile::from_file("SomeDamlModel.dar")?;
/// dar.apply(|archive| archive.accept(&mut visitor))?;
/// # Ok(())
/// # }
/// ```
#[allow(unused_variables)]
pub trait DamlElementVisitor {
    fn sort_elements(&self) -> bool {
        false
    }
    fn pre_visit_archive<'a>(&mut self, archive: &'a DamlArchive<'a>) {}
    fn post_visit_archive<'a>(&mut self, archive: &'a DamlArchive<'a>) {}
    fn pre_visit_package<'a>(&mut self, package: &'a DamlPackage<'a>) {}
    fn post_visit_package<'a>(&mut self, package: &'a DamlPackage<'a>) {}
    fn pre_visit_module<'a>(&mut self, module: &'a DamlModule<'a>) {}
    fn post_visit_module<'a>(&mut self, module: &'a DamlModule<'a>) {}
    fn pre_visit_def_type_syn<'a>(&mut self, def_type_syn: &'a DamlDefTypeSyn<'a>) {}
    fn post_visit_def_type_syn<'a>(&mut self, def_type_syn: &'a DamlDefTypeSyn<'a>) {}
    fn pre_visit_data<'a>(&mut self, data: &'a DamlData<'a>) {}
    fn post_visit_data<'a>(&mut self, data: &'a DamlData<'a>) {}
    fn pre_visit_template<'a>(&mut self, template: &'a DamlTemplate<'a>) {}
    fn post_visit_template<'a>(&mut self, template: &'a DamlTemplate<'a>) {}
    fn pre_visit_choice<'a>(&mut self, choice: &'a DamlChoice<'a>) {}
    fn post_visit_choice<'a>(&mut self, choice: &'a DamlChoice<'a>) {}
    fn pre_visit_record<'a>(&mut self, record: &'a DamlRecord<'a>) {}
    fn post_visit_record<'a>(&mut self, record: &'a DamlRecord<'a>) {}
    fn pre_visit_variant<'a>(&mut self, variant: &'a DamlVariant<'a>) {}
    fn post_visit_variant<'a>(&mut self, variant: &'a DamlVariant<'a>) {}
    fn pre_visit_enum<'a>(&mut self, data_enum: &'a DamlEnum<'a>) {}
    fn post_visit_enum<'a>(&mut self, data_enum: &'a DamlEnum<'a>) {}
    fn pre_visit_field<'a>(&mut self, field: &'a DamlField<'a>) {}
    fn post_visit_field<'a>(&mut self, field: &'a DamlField<'a>) {}
    fn pre_visit_type<'a>(&mut self, ty: &'a DamlType<'a>) {}
    fn post_visit_type<'a>(&mut self, ty: &'a DamlType<'a>) {}
    fn pre_visit_type_var<'a>(&mut self, type_var: &'a DamlTypeVarWithKind<'a>) {}
    fn post_visit_type_var<'a>(&mut self, type_var: &'a DamlTypeVarWithKind<'a>) {}
    fn pre_visit_kind(&mut self, kind: &DamlKind) {}
    fn post_visit_kind(&mut self, kind: &DamlKind) {}
    fn pre_visit_arrow(&mut self, arrow: &DamlArrow) {}
    fn post_visit_arrow(&mut self, arrow: &DamlArrow) {}
    fn pre_visit_var<'a>(&mut self, var: &'a DamlVar<'a>) {}
    fn post_visit_var<'a>(&mut self, var: &'a DamlVar<'a>) {}
    fn pre_visit_forall<'a>(&mut self, forall: &'a DamlForall<'a>) {}
    fn post_visit_forall<'a>(&mut self, forall: &'a DamlForall<'a>) {}
    fn pre_visit_struct<'a>(&mut self, tuple: &'a DamlStruct<'a>) {}
    fn post_visit_struct<'a>(&mut self, tuple: &'a DamlStruct<'a>) {}
    fn pre_visit_syn<'a>(&mut self, syn: &'a DamlSyn<'a>) {}
    fn post_visit_syn<'a>(&mut self, syn: &'a DamlSyn<'a>) {}
    fn pre_visit_tycon<'a>(&mut self, tycon: &'a DamlTyCon<'a>) {}
    fn post_visit_tycon<'a>(&mut self, tycon: &'a DamlTyCon<'a>) {}
    fn pre_visit_tycon_name<'a>(&mut self, tycon_name: &'a DamlTyConName<'a>) {}
    fn post_visit_tycon_name<'a>(&mut self, tycon_name: &'a DamlTyConName<'a>) {}
    fn pre_visit_local_tycon<'a>(&mut self, local_tycon: &'a DamlLocalTyCon<'a>) {}
    fn post_visit_local_tycon<'a>(&mut self, local_tycon: &'a DamlLocalTyCon<'a>) {}
    fn pre_visit_non_local_tycon<'a>(&mut self, non_local_tycon: &'a DamlNonLocalTyCon<'a>) {}
    fn post_visit_non_local_tycon<'a>(&mut self, non_local_tycon: &'a DamlNonLocalTyCon<'a>) {}
    fn pre_visit_absolute_tycon<'a>(&mut self, absolute_tycon: &'a DamlAbsoluteTyCon<'a>) {}
    fn post_visit_absolute_tycon<'a>(&mut self, absolute_tycon: &'a DamlAbsoluteTyCon<'a>) {}
    fn pre_visit_def_key(&mut self, def_key: &DamlDefKey<'_>) {}
    fn post_visit_def_key(&mut self, def_key: &DamlDefKey<'_>) {}

    #[cfg(feature = "full")]
    fn pre_visit_def_value<'a>(&mut self, def_value: &'a DamlDefValue<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_def_value<'a>(&mut self, def_value: &'a DamlDefValue<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_expr<'a>(&mut self, expr: &'a DamlExpr<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_expr<'a>(&mut self, expr: &'a DamlExpr<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_value_name<'a>(&mut self, value_name: &'a DamlValueName<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_local_value_name<'a>(&mut self, local_value_name: &'a DamlLocalValueName<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_local_value_name<'a>(&mut self, local_value_name: &'a DamlLocalValueName<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_non_local_value_name<'a>(&mut self, non_local_value_name: &'a DamlNonLocalValueName<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_non_local_value_name<'a>(&mut self, non_local_value_name: &'a DamlNonLocalValueName<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_value_name<'a>(&mut self, value_name: &'a DamlValueName<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_builtin_function(&mut self, builtin: &DamlBuiltinFunction) {}
    #[cfg(feature = "full")]
    fn post_visit_builtin_function(&mut self, builtin: &DamlBuiltinFunction) {}
    #[cfg(feature = "full")]
    fn pre_visit_prim_con(&mut self, prim_con: DamlPrimCon) {}
    #[cfg(feature = "full")]
    fn post_visit_prim_con(&mut self, prim_con: DamlPrimCon) {}
    #[cfg(feature = "full")]
    fn pre_visit_prim_lit(&mut self, prim_lit: &DamlPrimLit<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_prim_lit(&mut self, prim_lit: &DamlPrimLit<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_rounding_mode(&mut self, rounding_mode: &RoundingMode) {}
    #[cfg(feature = "full")]
    fn post_visit_rounding_mode(&mut self, rounding_mode: &RoundingMode) {}
    #[cfg(feature = "full")]
    fn pre_visit_rec_con(&mut self, rec_con: &DamlRecCon<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_rec_con(&mut self, rec_con: &DamlRecCon<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_field_with_expr(&mut self, field_with_expr: &DamlFieldWithExpr<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_field_with_expr(&mut self, field_with_expr: &DamlFieldWithExpr<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_rec_proj(&mut self, rec_proj: &DamlRecProj<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_rec_proj(&mut self, rec_proj: &DamlRecProj<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_rec_upd(&mut self, rec_upd: &DamlRecUpd<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_rec_upd(&mut self, rec_upd: &DamlRecUpd<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_variant_con(&mut self, variant_con: &DamlVariantCon<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_variant_con(&mut self, variant_con: &DamlVariantCon<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_enum_con(&mut self, enum_con: &DamlEnumCon<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_enum_con(&mut self, enum_con: &DamlEnumCon<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_struct_con(&mut self, struct_con: &DamlStructCon<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_struct_con(&mut self, struct_con: &DamlStructCon<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_struct_proj(&mut self, struct_proj: &DamlStructProj<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_struct_proj(&mut self, struct_proj: &DamlStructProj<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_struct_upd(&mut self, struct_upd: &DamlStructUpd<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_struct_upd(&mut self, struct_upd: &DamlStructUpd<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_app(&mut self, app: &DamlApp<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_app(&mut self, app: &DamlApp<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_ty_app(&mut self, ty_app: &DamlTyApp<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_ty_app(&mut self, ty_app: &DamlTyApp<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_abs(&mut self, abs: &DamlAbs<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_abs(&mut self, abs: &DamlAbs<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_var_with_type(&mut self, var_with_type: &DamlVarWithType<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_var_with_type(&mut self, var_with_type: &DamlVarWithType<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_ty_abs(&mut self, ty_abs: &DamlTyAbs<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_ty_abs(&mut self, ty_abs: &DamlTyAbs<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_case(&mut self, case: &DamlCase<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_case(&mut self, case: &DamlCase<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_block(&mut self, block: &DamlBlock<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_block(&mut self, block: &DamlBlock<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_binding(&mut self, binding: &DamlBinding<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_binding(&mut self, binding: &DamlBinding<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_cons(&mut self, cons: &DamlCons<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_cons(&mut self, cons: &DamlCons<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_optional_some(&mut self, optional_some: &DamlOptionalSome<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_optional_some(&mut self, optional_some: &DamlOptionalSome<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_to_any(&mut self, to_any: &DamlToAny<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_to_any(&mut self, to_any: &DamlToAny<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_from_any(&mut self, from_any: &DamlFromAny<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_from_any(&mut self, from_any: &DamlFromAny<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_update(&mut self, update: &DamlUpdate<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_update(&mut self, update: &DamlUpdate<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_pure(&mut self, pure: &DamlPure<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_pure(&mut self, pure: &DamlPure<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_create(&mut self, create: &DamlCreate<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_create(&mut self, create: &DamlCreate<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_exercise(&mut self, exercise: &DamlExercise<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_exercise(&mut self, exercise: &DamlExercise<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_exercise_by_key(&mut self, exercise_by_key: &DamlExerciseByKey<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_exercise_by_key(&mut self, exercise_by_key: &DamlExerciseByKey<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_fetch(&mut self, fetch: &DamlFetch<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_fetch(&mut self, fetch: &DamlFetch<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_try_catch(&mut self, try_catch: &DamlTryCatch<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_try_catch(&mut self, try_catch: &DamlTryCatch<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_retrieve_by_key(&mut self, retrieve_by_key: &DamlRetrieveByKey<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_retrieve_by_key(&mut self, retrieve_by_key: &DamlRetrieveByKey<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_update_embed_expr(&mut self, update_embed_expr: &DamlUpdateEmbedExpr<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_update_embed_expr(&mut self, update_embed_expr: &DamlUpdateEmbedExpr<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_scenario(&mut self, scenario: &DamlScenario<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_scenario(&mut self, scenario: &DamlScenario<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_commit(&mut self, commit: &DamlCommit<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_commit(&mut self, commit: &DamlCommit<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_scenario_embed_expr(&mut self, scenario_embed_expr: &DamlScenarioEmbedExpr<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_scenario_embed_expr(&mut self, scenario_embed_expr: &DamlScenarioEmbedExpr<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_case_alt(&mut self, case_alt: &DamlCaseAlt<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_case_alt(&mut self, case_alt: &DamlCaseAlt<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_case_alt_sum(&mut self, case_alt_sum: &DamlCaseAltSum<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_case_alt_sum(&mut self, case_alt_sum: &DamlCaseAltSum<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_case_alt_variant(&mut self, case_alt_variant: &DamlCaseAltVariant<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_case_alt_variant(&mut self, case_alt_variant: &DamlCaseAltVariant<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_case_alt_cons(&mut self, case_alt_cons: &DamlCaseAltCons<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_case_alt_cons(&mut self, case_alt_cons: &DamlCaseAltCons<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_case_alt_opt_some(&mut self, case_alt_opt_some: &DamlCaseAltOptionalSome<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_case_alt_opt_some(&mut self, case_alt_opt_some: &DamlCaseAltOptionalSome<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_case_alt_enum(&mut self, case_alt_enum: &DamlCaseAltEnum<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_case_alt_enum(&mut self, case_alt_enum: &DamlCaseAltEnum<'_>) {}

    #[cfg(feature = "full")]
    fn pre_visit_to_any_exception(&mut self, to_any_exception: &DamlToAnyException<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_to_any_exception(&mut self, to_any_exception: &DamlToAnyException<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_from_any_exception(&mut self, from_any_exception: &DamlFromAnyException<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_from_any_exception(&mut self, from_any_exception: &DamlFromAnyException<'_>) {}
    #[cfg(feature = "full")]
    fn pre_visit_throw(&mut self, throw: &DamlThrow<'_>) {}
    #[cfg(feature = "full")]
    fn post_visit_throw(&mut self, throw: &DamlThrow<'_>) {}
}
