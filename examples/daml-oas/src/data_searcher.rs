use crate::choice_event_extractor::ChoiceEventExtractor;
use crate::filter::ChoiceFilter;
use daml::lf::element::{DamlArchive, DamlChoice, DamlData, DamlField, DamlTemplate, DamlTyCon, DamlType};

/// Search for any usage of a `DamlData` needle in a `DamlTemplate` haystack.
pub struct DamlEntitySearcher<'v> {
    archive: &'v DamlArchive<'v>,
    needle: &'v DamlData<'v>,
}

impl<'v> DamlEntitySearcher<'v> {
    pub const fn new(archive: &'v DamlArchive<'v>, needle: &'v DamlData<'v>) -> Self {
        Self {
            archive,
            needle,
        }
    }

    /// Search for usage of the `DamlData` needle in the given `DamlTemplate` haystack.
    pub fn search_template(&self, template: &DamlTemplate<'_>, choice_filter: &ChoiceFilter) -> bool {
        self.check_template(template) || self.check_choices(template, choice_filter)
    }

    fn check_template(&self, template: &DamlTemplate<'_>) -> bool {
        self.check_template_name(template) || self.check_template_key(template) || self.check_template_fields(template)
    }

    fn check_template_name(&self, template: &DamlTemplate<'_>) -> bool {
        template == self.needle
    }

    fn check_template_fields(&self, template: &DamlTemplate<'_>) -> bool {
        self.check_fields(template.fields())
    }

    fn check_template_key(&self, template: &DamlTemplate<'_>) -> bool {
        template.key().map_or(false, |key| self.check_type(key.ty()))
    }

    fn check_choices(&self, template: &DamlTemplate<'_>, choice_filter: &ChoiceFilter) -> bool {
        template.choices().iter().any(|choice| self.check_choice(choice, choice_filter) || self.check_events(choice))
    }

    /// Check if the target data is in scope of choice filer and if so check it and it's arguments.
    fn check_choice(&self, choice: &DamlChoice<'_>, choice_filter: &ChoiceFilter) -> bool {
        self.is_choice_in_scope(choice_filter)
            && (self.check_choice_name(choice)
                || self.check_choice_return_type(choice)
                || self.check_choice_fields(choice))
    }

    fn check_choice_name(&self, choice: &DamlChoice<'_>) -> bool {
        choice.package_id() == self.needle.package_id()
            && choice.module_path().zip(self.needle.module_path()).all(|(x, y)| x == y)
            && choice.name() == self.needle.name()
    }

    fn check_choice_return_type(&self, choice: &DamlChoice<'_>) -> bool {
        self.check_type(choice.return_type())
    }

    fn check_choice_fields(&self, choice: &DamlChoice<'_>) -> bool {
        self.check_fields(choice.fields())
    }

    /// Check the events which may be emitted by this choice.
    ///
    /// Note that we can skip archived events as we don't include payload schema for those.
    fn check_events(&self, choice: &DamlChoice<'_>) -> bool {
        self.archive
            .extract_choice_events(
                self.needle.package_id(),
                &self.needle.module_path().collect::<Vec<_>>(),
                self.needle.name(),
                choice,
            )
            .created()
            .any(|c| self.check_event(c))
    }

    fn check_event(&self, tycon: &DamlTyCon<'_>) -> bool {
        self.check_tycon_name(tycon) || self.check_tycon(tycon)
    }

    fn check_tycon_name(&self, tycon: &DamlTyCon<'_>) -> bool {
        tycon.tycon() == self.needle
    }

    fn check_data(&self, data: &DamlData<'_>) -> bool {
        match data {
            DamlData::Template(template) => self.check_fields(template.fields()),
            DamlData::Record(record) => self.check_fields(record.fields()),
            DamlData::Variant(variant) => self.check_fields(variant.fields()),
            DamlData::Enum(_) => false,
        }
    }

    fn check_fields(&self, fields: &[DamlField<'_>]) -> bool {
        fields.iter().any(|f| self.check_field(f))
    }

    fn check_field(&self, field: &DamlField<'_>) -> bool {
        self.check_type(field.ty())
    }

    fn check_type(&self, ty: &DamlType<'_>) -> bool {
        match ty {
            DamlType::TyCon(tycon) | DamlType::BoxedTyCon(tycon) => self.check_tycon(tycon),
            DamlType::ContractId(ty) => ty.as_deref().map_or(false, |ty| self.check_type(ty)),
            DamlType::List(args) | DamlType::TextMap(args) | DamlType::GenMap(args) | DamlType::Optional(args) =>
                args.iter().any(|ty| self.check_type(ty)),
            _ => false,
        }
    }

    fn check_tycon(&self, tycon: &DamlTyCon<'_>) -> bool {
        if tycon.tycon() == self.needle {
            true
        } else {
            self.archive
                .data_by_tycon(tycon)
                .map_or_else(|| panic!("data_by_tycon returned None for {:?}", tycon), |data| self.check_data(data))
        }
    }

    /// Check if the target data is in the scope of the choice filer.
    fn is_choice_in_scope(&self, choice_filter: &ChoiceFilter) -> bool {
        match choice_filter {
            ChoiceFilter::None => false,
            ChoiceFilter::All => true,
            ChoiceFilter::Selected(selected) =>
                selected.iter().any(|selected_choice| selected_choice == self.needle.name()),
        }
    }
}
