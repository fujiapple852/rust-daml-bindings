use crate::element::{
    DamlAbsoluteDataRef, DamlArchive, DamlArrow, DamlChoice, DamlData, DamlDataRef, DamlEnum, DamlField, DamlKind,
    DamlLocalDataRef, DamlModule, DamlNonLocalDataRef, DamlPackage, DamlRecord, DamlTemplate, DamlType, DamlTypeVar,
    DamlVar, DamlVariant,
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
    fn pre_visit_type_var<'a>(&mut self, type_var: &'a DamlTypeVar<'a>) {}
    fn post_visit_type_var<'a>(&mut self, type_var: &'a DamlTypeVar<'a>) {}
    fn pre_visit_kind(&mut self, kind: &DamlKind) {}
    fn post_visit_kind(&mut self, kind: &DamlKind) {}
    fn pre_visit_arrow(&mut self, arrow: &DamlArrow) {}
    fn post_visit_arrow(&mut self, arrow: &DamlArrow) {}
    fn pre_visit_var<'a>(&mut self, var: &'a DamlVar<'a>) {}
    fn post_visit_var<'a>(&mut self, var: &'a DamlVar<'a>) {}
    fn pre_visit_data_ref<'a>(&mut self, data_ref: &'a DamlDataRef<'a>) {}
    fn post_visit_data_ref<'a>(&mut self, data_ref: &'a DamlDataRef<'a>) {}
    fn pre_visit_local_data_ref<'a>(&mut self, local_data_ref: &'a DamlLocalDataRef<'a>) {}
    fn post_visit_local_data_ref<'a>(&mut self, local_data_ref: &'a DamlLocalDataRef<'a>) {}
    fn pre_visit_non_local_data_ref<'a>(&mut self, non_local_data_ref: &'a DamlNonLocalDataRef<'a>) {}
    fn post_visit_non_local_data_ref<'a>(&mut self, non_local_data_ref: &'a DamlNonLocalDataRef<'a>) {}
    fn pre_visit_absolute_data_ref<'a>(&mut self, absolute_data_ref: &'a DamlAbsoluteDataRef<'a>) {}
    fn post_visit_absolute_data_ref<'a>(&mut self, absolute_data_ref: &'a DamlAbsoluteDataRef<'a>) {}
}
