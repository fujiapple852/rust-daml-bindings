use std::collections::HashSet;

use daml::lf::element::{
    DamlArchive, DamlChoice, DamlData, DamlElementVisitor, DamlField, DamlTemplate, DamlTyCon, DamlType,
    DamlVisitableElement,
};

use crate::choice_event_extractor::ChoiceEventExtractor;
use crate::filter::ChoiceFilter;

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
            .any(|c| c.tycon().data_name() == self.needle.name())
    }

    fn check_fields(&self, fields: &[DamlField<'_>]) -> bool {
        fields.iter().any(|field| self.check_type(field.ty()))
    }

    fn check_type(&self, ty: &DamlType<'_>) -> bool {
        let mut vis = DataVisitor::new(self.archive, self.needle);
        ty.accept(&mut vis);
        vis.found
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

struct DataVisitor<'v> {
    archive: &'v DamlArchive<'v>,
    needle: &'v DamlData<'v>,
    visited: HashSet<String>,
    found: bool,
}

impl<'v> DataVisitor<'v> {
    pub fn new(archive: &'v DamlArchive<'v>, needle: &'v DamlData<'_>) -> Self {
        Self {
            archive,
            needle,
            visited: HashSet::default(),
            found: false,
        }
    }
}

impl DamlElementVisitor for DataVisitor<'_> {
    fn pre_visit_tycon<'a>(&mut self, tycon: &'a DamlTyCon<'a>) {
        let name = tycon.tycon().to_string();
        if !self.visited.contains(&name) {
            self.visited.insert(name);
            if tycon.tycon() == self.needle {
                self.found = true;
            }
            let data = self
                .archive
                .data_by_tycon(tycon)
                .unwrap_or_else(|| panic!("data_by_tycon returned None for {:?}", tycon));
            data.accept(self)
        }
    }
}
