#[cfg(feature = "full")]
use crate::element::daml_expr::DamlScenario;
#[cfg(feature = "full")]
use crate::element::{
    DamlAbs, DamlApp, DamlBinding, DamlBlock, DamlBuiltinFunction, DamlCase, DamlCaseAlt, DamlCaseAltCons,
    DamlCaseAltEnum, DamlCaseAltOptionalSome, DamlCaseAltSum, DamlCaseAltVariant, DamlCommit, DamlCons, DamlCreate,
    DamlDefValue, DamlEnumCon, DamlExercise, DamlExerciseByKey, DamlExpr, DamlFetch, DamlFieldWithExpr, DamlFromAny,
    DamlFromAnyException, DamlOptionalSome, DamlPrimCon, DamlPrimLit, DamlPure, DamlRecCon, DamlRecProj, DamlRecUpd,
    DamlRetrieveByKey, DamlScenarioEmbedExpr, DamlStructCon, DamlStructProj, DamlStructUpd, DamlThrow, DamlToAny,
    DamlToAnyException, DamlTryCatch, DamlTyAbs, DamlTyApp, DamlUpdate, DamlUpdateEmbedExpr, DamlValueName,
    DamlVarWithType, DamlVariantCon,
};
use crate::element::{
    DamlAbsoluteTyCon, DamlArchive, DamlArrow, DamlChoice, DamlData, DamlDefException, DamlDefKey, DamlDefTypeSyn,
    DamlEnum, DamlField, DamlForall, DamlKind, DamlLocalTyCon, DamlModule, DamlNonLocalTyCon, DamlPackage, DamlRecord,
    DamlStruct, DamlSyn, DamlTemplate, DamlTyCon, DamlTyConName, DamlType, DamlTypeVarWithKind, DamlVar, DamlVariant,
};

pub trait DamlVisitableElement<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor);
}

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
    fn pre_visit_def_exception<'a>(&mut self, def_exception: &'a DamlDefException<'a>) {}
    fn post_visit_def_exception<'a>(&mut self, def_exception: &'a DamlDefException<'a>) {}
    fn pre_visit_def_key<'a>(&mut self, def_key: &DamlDefKey<'a>) {}
    fn post_visit_def_key<'a>(&mut self, def_key: &DamlDefKey<'a>) {}

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
    fn pre_visit_prim_lit<'a>(&mut self, prim_lit: &DamlPrimLit<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_prim_lit<'a>(&mut self, prim_lit: &DamlPrimLit<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_rec_con<'a>(&mut self, rec_con: &DamlRecCon<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_rec_con<'a>(&mut self, rec_con: &DamlRecCon<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_field_with_expr<'a>(&mut self, field_with_expr: &DamlFieldWithExpr<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_field_with_expr<'a>(&mut self, field_with_expr: &DamlFieldWithExpr<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_rec_proj<'a>(&mut self, rec_proj: &DamlRecProj<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_rec_proj<'a>(&mut self, rec_proj: &DamlRecProj<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_rec_upd<'a>(&mut self, rec_upd: &DamlRecUpd<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_rec_upd<'a>(&mut self, rec_upd: &DamlRecUpd<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_variant_con<'a>(&mut self, variant_con: &DamlVariantCon<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_variant_con<'a>(&mut self, variant_con: &DamlVariantCon<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_enum_con<'a>(&mut self, enum_con: &DamlEnumCon<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_enum_con<'a>(&mut self, enum_con: &DamlEnumCon<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_struct_con<'a>(&mut self, struct_con: &DamlStructCon<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_struct_con<'a>(&mut self, struct_con: &DamlStructCon<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_struct_proj<'a>(&mut self, struct_proj: &DamlStructProj<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_struct_proj<'a>(&mut self, struct_proj: &DamlStructProj<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_struct_upd<'a>(&mut self, struct_upd: &DamlStructUpd<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_struct_upd<'a>(&mut self, struct_upd: &DamlStructUpd<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_app<'a>(&mut self, app: &DamlApp<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_app<'a>(&mut self, app: &DamlApp<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_ty_app<'a>(&mut self, ty_app: &DamlTyApp<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_ty_app<'a>(&mut self, ty_app: &DamlTyApp<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_abs<'a>(&mut self, abs: &DamlAbs<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_abs<'a>(&mut self, abs: &DamlAbs<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_var_with_type<'a>(&mut self, var_with_type: &DamlVarWithType<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_var_with_type<'a>(&mut self, var_with_type: &DamlVarWithType<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_ty_abs<'a>(&mut self, ty_abs: &DamlTyAbs<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_ty_abs<'a>(&mut self, ty_abs: &DamlTyAbs<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_case<'a>(&mut self, case: &DamlCase<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_case<'a>(&mut self, case: &DamlCase<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_block<'a>(&mut self, block: &DamlBlock<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_block<'a>(&mut self, block: &DamlBlock<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_binding<'a>(&mut self, binding: &DamlBinding<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_binding<'a>(&mut self, binding: &DamlBinding<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_cons<'a>(&mut self, cons: &DamlCons<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_cons<'a>(&mut self, cons: &DamlCons<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_optional_some<'a>(&mut self, optional_some: &DamlOptionalSome<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_optional_some<'a>(&mut self, optional_some: &DamlOptionalSome<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_to_any<'a>(&mut self, to_any: &DamlToAny<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_to_any<'a>(&mut self, to_any: &DamlToAny<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_from_any<'a>(&mut self, from_any: &DamlFromAny<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_from_any<'a>(&mut self, from_any: &DamlFromAny<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_update<'a>(&mut self, update: &DamlUpdate<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_update<'a>(&mut self, update: &DamlUpdate<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_pure<'a>(&mut self, pure: &DamlPure<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_pure<'a>(&mut self, pure: &DamlPure<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_create<'a>(&mut self, create: &DamlCreate<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_create<'a>(&mut self, create: &DamlCreate<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_exercise<'a>(&mut self, exercise: &DamlExercise<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_exercise<'a>(&mut self, exercise: &DamlExercise<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_exercise_by_key<'a>(&mut self, exercise_by_key: &DamlExerciseByKey<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_exercise_by_key<'a>(&mut self, exercise_by_key: &DamlExerciseByKey<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_try_catch<'a>(&mut self, try_catch: &DamlTryCatch<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_try_catch<'a>(&mut self, try_catch: &DamlTryCatch<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_fetch<'a>(&mut self, fetch: &DamlFetch<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_fetch<'a>(&mut self, fetch: &DamlFetch<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_retrieve_by_key<'a>(&mut self, retrieve_by_key: &DamlRetrieveByKey<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_retrieve_by_key<'a>(&mut self, retrieve_by_key: &DamlRetrieveByKey<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_update_embed_expr<'a>(&mut self, update_embed_expr: &DamlUpdateEmbedExpr<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_update_embed_expr<'a>(&mut self, update_embed_expr: &DamlUpdateEmbedExpr<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_scenario<'a>(&mut self, scenario: &DamlScenario<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_scenario<'a>(&mut self, scenario: &DamlScenario<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_commit<'a>(&mut self, commit: &DamlCommit<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_commit<'a>(&mut self, commit: &DamlCommit<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_scenario_embed_expr<'a>(&mut self, scenario_embed_expr: &DamlScenarioEmbedExpr<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_scenario_embed_expr<'a>(&mut self, scenario_embed_expr: &DamlScenarioEmbedExpr<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_case_alt<'a>(&mut self, case_alt: &DamlCaseAlt<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_case_alt<'a>(&mut self, case_alt: &DamlCaseAlt<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_case_alt_sum<'a>(&mut self, case_alt_sum: &DamlCaseAltSum<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_case_alt_sum<'a>(&mut self, case_alt_sum: &DamlCaseAltSum<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_case_alt_variant<'a>(&mut self, case_alt_variant: &DamlCaseAltVariant<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_case_alt_variant<'a>(&mut self, case_alt_variant: &DamlCaseAltVariant<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_case_alt_cons<'a>(&mut self, case_alt_cons: &DamlCaseAltCons<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_case_alt_cons<'a>(&mut self, case_alt_cons: &DamlCaseAltCons<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_case_alt_opt_some<'a>(&mut self, case_alt_opt_some: &DamlCaseAltOptionalSome<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_case_alt_opt_some<'a>(&mut self, case_alt_opt_some: &DamlCaseAltOptionalSome<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_case_alt_enum<'a>(&mut self, case_alt_enum: &DamlCaseAltEnum<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_case_alt_enum<'a>(&mut self, case_alt_enum: &DamlCaseAltEnum<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_to_any_exception<'a>(&mut self, to_any_exception: &DamlToAnyException<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_to_any_exception<'a>(&mut self, to_any_exception: &DamlToAnyException<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_from_any_exception<'a>(&mut self, from_any_exception: &DamlFromAnyException<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_from_any_exception<'a>(&mut self, from_any_exception: &DamlFromAnyException<'a>) {}
    #[cfg(feature = "full")]
    fn pre_visit_throw<'a>(&mut self, throw: &DamlThrow<'a>) {}
    #[cfg(feature = "full")]
    fn post_visit_throw<'a>(&mut self, throw: &DamlThrow<'a>) {}
}
