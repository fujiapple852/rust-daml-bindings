use crate::element::daml_field::DamlField;
use crate::element::visitor::DamlElementVisitor;
#[cfg(feature = "full")]
use crate::element::{DamlDefKey, DamlExpr, DamlPrimLit};
use crate::element::{DamlType, DamlTypeVarWithKind, DamlVisitableElement};
use crate::owned::ToStatic;
use serde::Serialize;
use std::borrow::Cow;

#[derive(Debug, Serialize, Clone)]
pub enum DamlData<'a> {
    Template(Box<DamlTemplate<'a>>),
    Record(DamlRecord<'a>),
    Variant(DamlVariant<'a>),
    Enum(DamlEnum<'a>),
}

impl<'a> DamlData<'a> {
    /// The name of this data type.
    pub fn name(&self) -> &str {
        match self {
            DamlData::Record(record) => &record.name,
            DamlData::Template(template) => &template.name,
            DamlData::Variant(variant) => &variant.name,
            DamlData::Enum(data_enum) => &data_enum.name,
        }
    }

    /// The fields of this data type.
    pub fn fields(&self) -> &[DamlField<'_>] {
        match self {
            DamlData::Record(record) => &record.fields,
            DamlData::Template(template) => &template.fields,
            DamlData::Variant(variant) => &variant.fields,
            DamlData::Enum(_) => &[],
        }
    }

    /// The type arguments applied to this data type.
    pub fn type_arguments(&self) -> &[DamlTypeVarWithKind<'_>] {
        match self {
            DamlData::Record(record) => &record.type_arguments,
            DamlData::Template(_) => &[],
            DamlData::Variant(variant) => &variant.type_arguments,
            DamlData::Enum(data_enum) => &data_enum.type_arguments,
        }
    }

    /// Is this data type serializable?
    pub fn serializable(&self) -> bool {
        match self {
            DamlData::Record(record) => record.serializable,
            DamlData::Template(template) => template.serializable,
            DamlData::Variant(variant) => variant.serializable,
            DamlData::Enum(data_enum) => data_enum.serializable,
        }
    }

    /// The name of this data type.
    ///
    /// This is a clone of a `Cow<str>` which is cheap for the borrowed case used within the library.
    #[doc(hidden)]
    pub(crate) fn name_clone(&self) -> Cow<'a, str> {
        match self {
            DamlData::Record(record) => record.name.clone(),
            DamlData::Template(template) => template.name.clone(),
            DamlData::Variant(variant) => variant.name.clone(),
            DamlData::Enum(data_enum) => data_enum.name.clone(),
        }
    }
}

impl<'a> DamlVisitableElement<'a> for DamlData<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_data(self);
        match self {
            DamlData::Record(record) => record.accept(visitor),
            DamlData::Template(template) => template.accept(visitor),
            DamlData::Variant(variant) => variant.accept(visitor),
            DamlData::Enum(data_enum) => data_enum.accept(visitor),
        }
        visitor.post_visit_data(self);
    }
}

impl ToStatic for DamlData<'_> {
    type Static = DamlData<'static>;

    fn to_static(&self) -> Self::Static {
        match self {
            DamlData::Record(record) => DamlData::Record(record.to_static()),
            DamlData::Template(template) => DamlData::Template(Box::new(template.as_ref().to_static())),
            DamlData::Variant(variant) => DamlData::Variant(variant.to_static()),
            DamlData::Enum(data_enum) => DamlData::Enum(data_enum.to_static()),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlTemplate<'a> {
    name: Cow<'a, str>,
    package_id: Cow<'a, str>,
    module_path: Vec<Cow<'a, str>>,
    fields: Vec<DamlField<'a>>,
    choices: Vec<DamlChoice<'a>>,
    param: Cow<'a, str>,
    #[cfg(feature = "full")]
    precond: Option<DamlExpr<'a>>,
    #[cfg(feature = "full")]
    signatories: DamlExpr<'a>,
    #[cfg(feature = "full")]
    agreement: DamlExpr<'a>,
    #[cfg(feature = "full")]
    observers: DamlExpr<'a>,
    #[cfg(feature = "full")]
    key: Option<DamlDefKey<'a>>,
    serializable: bool,
}

impl<'a> DamlTemplate<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: Cow<'a, str>,
        package_id: Cow<'a, str>,
        module_path: Vec<Cow<'a, str>>,
        fields: Vec<DamlField<'a>>,
        choices: Vec<DamlChoice<'a>>,
        param: Cow<'a, str>,
        #[cfg(feature = "full")] precond: Option<DamlExpr<'a>>,
        #[cfg(feature = "full")] signatories: DamlExpr<'a>,
        #[cfg(feature = "full")] agreement: DamlExpr<'a>,
        #[cfg(feature = "full")] observers: DamlExpr<'a>,
        #[cfg(feature = "full")] key: Option<DamlDefKey<'a>>,
        serializable: bool,
    ) -> Self {
        Self {
            name,
            package_id,
            module_path,
            fields,
            choices,
            param,
            #[cfg(feature = "full")]
            precond,
            #[cfg(feature = "full")]
            signatories,
            #[cfg(feature = "full")]
            agreement,
            #[cfg(feature = "full")]
            observers,
            #[cfg(feature = "full")]
            key,
            serializable,
        }
    }

    pub fn new_with_defaults(
        name: Cow<'a, str>,
        package_id: Cow<'a, str>,
        module_path: Vec<Cow<'a, str>>,
        fields: Vec<DamlField<'a>>,
    ) -> Self {
        Self {
            name,
            package_id,
            module_path,
            fields,
            choices: vec![],
            param: Cow::default(),
            #[cfg(feature = "full")]
            precond: None,
            #[cfg(feature = "full")]
            signatories: DamlExpr::Nil(DamlType::List(vec![DamlType::Party])),
            #[cfg(feature = "full")]
            agreement: DamlExpr::PrimLit(DamlPrimLit::Text(Cow::default())),
            #[cfg(feature = "full")]
            observers: DamlExpr::Nil(DamlType::List(vec![DamlType::Party])),
            #[cfg(feature = "full")]
            key: None,
            serializable: true,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn package_id(&self) -> &str {
        &self.package_id
    }

    pub fn module_path(&self) -> impl Iterator<Item = &str> {
        self.module_path.iter().map(AsRef::as_ref)
    }

    pub fn fields(&self) -> &[DamlField<'a>] {
        &self.fields
    }

    pub fn choices(&self) -> &[DamlChoice<'a>] {
        &self.choices
    }

    pub fn param(&self) -> &str {
        &self.param
    }

    #[cfg(feature = "full")]
    pub fn precond(&self) -> Option<&DamlExpr<'a>> {
        self.precond.as_ref()
    }

    #[cfg(feature = "full")]
    pub fn signatories(&self) -> &DamlExpr<'a> {
        &self.signatories
    }

    #[cfg(feature = "full")]
    pub fn agreement(&self) -> &DamlExpr<'a> {
        &self.agreement
    }

    #[cfg(feature = "full")]
    pub fn observers(&self) -> &DamlExpr<'a> {
        &self.observers
    }

    #[cfg(feature = "full")]
    pub fn key(&self) -> Option<&DamlDefKey<'a>> {
        self.key.as_ref()
    }

    pub const fn serializable(&self) -> bool {
        self.serializable
    }
}

impl<'a> DamlVisitableElement<'a> for DamlTemplate<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_template(self);
        self.fields.iter().for_each(|field| field.accept(visitor));
        self.choices.iter().for_each(|choice| choice.accept(visitor));
        #[cfg(feature = "full")]
        self.precond.iter().for_each(|pre| pre.accept(visitor));
        #[cfg(feature = "full")]
        self.signatories.accept(visitor);
        #[cfg(feature = "full")]
        self.agreement.accept(visitor);
        #[cfg(feature = "full")]
        self.observers.accept(visitor);
        #[cfg(feature = "full")]
        self.key.iter().for_each(|k| k.accept(visitor));
        visitor.post_visit_template(self);
    }
}

impl ToStatic for DamlTemplate<'_> {
    type Static = DamlTemplate<'static>;

    fn to_static(&self) -> Self::Static {
        DamlTemplate::new(
            self.name.to_static(),
            self.package_id.to_static(),
            self.module_path.iter().map(ToStatic::to_static).collect(),
            self.fields.iter().map(DamlField::to_static).collect(),
            self.choices.iter().map(DamlChoice::to_static).collect(),
            self.param.to_static(),
            #[cfg(feature = "full")]
            self.precond.as_ref().map(DamlExpr::to_static),
            #[cfg(feature = "full")]
            self.signatories.to_static(),
            #[cfg(feature = "full")]
            self.agreement.to_static(),
            #[cfg(feature = "full")]
            self.observers.to_static(),
            #[cfg(feature = "full")]
            self.key.as_ref().map(DamlDefKey::to_static),
            self.serializable,
        )
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlChoice<'a> {
    name: Cow<'a, str>,
    fields: Vec<DamlField<'a>>,
    return_type: DamlType<'a>,
    consuming: bool,
    self_binder: Cow<'a, str>,
    #[cfg(feature = "full")]
    update: DamlExpr<'a>,
    #[cfg(feature = "full")]
    controllers: DamlExpr<'a>,
    #[cfg(feature = "full")]
    observers: DamlExpr<'a>,
}

impl<'a> DamlChoice<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: Cow<'a, str>,
        fields: Vec<DamlField<'a>>,
        return_type: DamlType<'a>,
        consuming: bool,
        self_binder: Cow<'a, str>,
        #[cfg(feature = "full")] update: DamlExpr<'a>,
        #[cfg(feature = "full")] controllers: DamlExpr<'a>,
        #[cfg(feature = "full")] observers: DamlExpr<'a>,
    ) -> Self {
        Self {
            name,
            fields,
            return_type,
            consuming,
            self_binder,
            #[cfg(feature = "full")]
            update,
            #[cfg(feature = "full")]
            controllers,
            #[cfg(feature = "full")]
            observers,
        }
    }

    pub fn new_with_default(name: Cow<'a, str>, fields: Vec<DamlField<'a>>, return_type: DamlType<'a>) -> Self {
        Self {
            name,
            fields,
            return_type,
            consuming: false,
            self_binder: Cow::default(),
            #[cfg(feature = "full")]
            update: DamlExpr::Nil(DamlType::List(vec![DamlType::Party])),
            #[cfg(feature = "full")]
            controllers: DamlExpr::Nil(DamlType::List(vec![DamlType::Party])),
            #[cfg(feature = "full")]
            observers: DamlExpr::Nil(DamlType::List(vec![DamlType::Party])),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn fields(&self) -> &[DamlField<'a>] {
        &self.fields
    }

    pub const fn return_type(&self) -> &DamlType<'a> {
        &self.return_type
    }

    pub const fn consuming(&self) -> bool {
        self.consuming
    }

    pub fn self_binder(&self) -> &str {
        &self.self_binder
    }

    #[cfg(feature = "full")]
    pub fn update(&self) -> &DamlExpr<'a> {
        &self.update
    }

    #[cfg(feature = "full")]
    pub fn controllers(&self) -> &DamlExpr<'a> {
        &self.controllers
    }

    #[cfg(feature = "full")]
    pub fn observers(&self) -> &DamlExpr<'a> {
        &self.observers
    }
}

impl<'a> DamlVisitableElement<'a> for DamlChoice<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_choice(self);
        self.fields.iter().for_each(|field| field.accept(visitor));
        self.return_type.accept(visitor);
        #[cfg(feature = "full")]
        self.update.accept(visitor);
        #[cfg(feature = "full")]
        self.controllers.accept(visitor);
        #[cfg(feature = "full")]
        self.observers.accept(visitor);
        visitor.post_visit_choice(self);
    }
}

impl ToStatic for DamlChoice<'_> {
    type Static = DamlChoice<'static>;

    fn to_static(&self) -> Self::Static {
        DamlChoice::new(
            self.name.to_static(),
            self.fields.iter().map(DamlField::to_static).collect(),
            self.return_type.to_static(),
            self.consuming,
            self.self_binder.to_static(),
            #[cfg(feature = "full")]
            self.update.to_static(),
            #[cfg(feature = "full")]
            self.controllers.to_static(),
            #[cfg(feature = "full")]
            self.observers.to_static(),
        )
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlRecord<'a> {
    name: Cow<'a, str>,
    fields: Vec<DamlField<'a>>,
    type_arguments: Vec<DamlTypeVarWithKind<'a>>,
    serializable: bool,
}

impl<'a> DamlRecord<'a> {
    pub fn new(
        name: Cow<'a, str>,
        fields: Vec<DamlField<'a>>,
        type_arguments: Vec<DamlTypeVarWithKind<'a>>,
        serializable: bool,
    ) -> Self {
        Self {
            name,
            fields,
            type_arguments,
            serializable,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn fields(&self) -> &[DamlField<'a>] {
        &self.fields
    }

    pub fn type_arguments(&self) -> &[DamlTypeVarWithKind<'a>] {
        &self.type_arguments
    }

    pub const fn serializable(&self) -> bool {
        self.serializable
    }
}

impl<'a> DamlVisitableElement<'a> for DamlRecord<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_record(self);
        self.fields.iter().for_each(|field| field.accept(visitor));
        self.type_arguments.iter().for_each(|arg| arg.accept(visitor));
        visitor.post_visit_record(self);
    }
}

impl ToStatic for DamlRecord<'_> {
    type Static = DamlRecord<'static>;

    fn to_static(&self) -> Self::Static {
        DamlRecord::new(
            self.name.to_static(),
            self.fields.iter().map(DamlField::to_static).collect(),
            self.type_arguments.iter().map(DamlTypeVarWithKind::to_static).collect(),
            self.serializable,
        )
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlVariant<'a> {
    name: Cow<'a, str>,
    fields: Vec<DamlField<'a>>,
    type_arguments: Vec<DamlTypeVarWithKind<'a>>,
    serializable: bool,
}

impl<'a> DamlVariant<'a> {
    pub fn new(
        name: Cow<'a, str>,
        fields: Vec<DamlField<'a>>,
        type_arguments: Vec<DamlTypeVarWithKind<'a>>,
        serializable: bool,
    ) -> Self {
        Self {
            name,
            fields,
            type_arguments,
            serializable,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn fields(&self) -> &[DamlField<'a>] {
        &self.fields
    }

    pub fn type_arguments(&self) -> &[DamlTypeVarWithKind<'a>] {
        &self.type_arguments
    }

    pub const fn serializable(&self) -> bool {
        self.serializable
    }
}

impl<'a> DamlVisitableElement<'a> for DamlVariant<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_variant(self);
        self.fields.iter().for_each(|field| field.accept(visitor));
        self.type_arguments.iter().for_each(|arg| arg.accept(visitor));
        visitor.post_visit_variant(self);
    }
}

impl ToStatic for DamlVariant<'_> {
    type Static = DamlVariant<'static>;

    fn to_static(&self) -> Self::Static {
        DamlVariant::new(
            self.name.to_static(),
            self.fields.iter().map(DamlField::to_static).collect(),
            self.type_arguments.iter().map(DamlTypeVarWithKind::to_static).collect(),
            self.serializable,
        )
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DamlEnum<'a> {
    name: Cow<'a, str>,
    constructors: Vec<Cow<'a, str>>,
    type_arguments: Vec<DamlTypeVarWithKind<'a>>,
    serializable: bool,
}

impl<'a> DamlEnum<'a> {
    pub fn new(
        name: Cow<'a, str>,
        constructors: Vec<Cow<'a, str>>,
        type_arguments: Vec<DamlTypeVarWithKind<'a>>,
        serializable: bool,
    ) -> Self {
        Self {
            name,
            constructors,
            type_arguments,
            serializable,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn constructors(&self) -> impl Iterator<Item = &str> {
        self.constructors.iter().map(AsRef::as_ref)
    }

    pub fn type_arguments(&self) -> &[DamlTypeVarWithKind<'a>] {
        &self.type_arguments
    }

    pub const fn serializable(&self) -> bool {
        self.serializable
    }
}

impl<'a> DamlVisitableElement<'a> for DamlEnum<'a> {
    fn accept(&'a self, visitor: &'a mut impl DamlElementVisitor) {
        visitor.pre_visit_enum(self);
        self.type_arguments.iter().for_each(|arg| arg.accept(visitor));
        visitor.post_visit_enum(self);
    }
}

impl ToStatic for DamlEnum<'_> {
    type Static = DamlEnum<'static>;

    fn to_static(&self) -> Self::Static {
        DamlEnum::new(
            self.name.to_static(),
            self.constructors.iter().map(ToStatic::to_static).collect(),
            self.type_arguments.iter().map(DamlTypeVarWithKind::to_static).collect(),
            self.serializable,
        )
    }
}
