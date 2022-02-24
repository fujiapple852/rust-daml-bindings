use std::collections::HashSet;

use crate::common::ARCHIVE_CHOICE_NAME;
use bounded_static::ToBoundedStatic;
use daml::lf::element::{
    DamlArchive, DamlChoice, DamlCreate, DamlData, DamlElementVisitor, DamlExercise, DamlExerciseByKey, DamlTyCon,
    DamlValueName, DamlVisitableElement,
};

/// Extract the `ChoiceEvents` that can be produced by a given `DamlChoice`.
pub trait ChoiceEventExtractor {
    /// Extract all events which may be created as an effect of invoking a `DamlChoice`.
    fn extract_choice_events<S: AsRef<str>>(
        &self,
        package_id: &str,
        module: &[S],
        template: &str,
        choice: &DamlChoice<'_>,
    ) -> ChoiceEvents<'_>;
}

/// DOCME
pub struct ChoiceEvents<'a> {
    created: HashSet<DamlTyCon<'a>>,
    archived: HashSet<DamlTyCon<'a>>,
}

impl<'a> ChoiceEvents<'a> {
    pub const fn new(created: HashSet<DamlTyCon<'a>>, archived: HashSet<DamlTyCon<'a>>) -> Self {
        Self {
            created,
            archived,
        }
    }

    pub fn created(&self) -> impl Iterator<Item = &DamlTyCon<'_>> {
        self.created.iter()
    }

    pub fn archived(&self) -> impl Iterator<Item = &DamlTyCon<'_>> {
        self.archived.iter()
    }
}

impl<'a> ChoiceEventExtractor for DamlArchive<'a> {
    fn extract_choice_events<S: AsRef<str>>(
        &self,
        package_id: &str,
        module_path: &[S],
        template_name: &str,
        choice: &DamlChoice<'_>,
    ) -> ChoiceEvents<'_> {
        let mut extractor = ChoiceEventVisitor {
            archive: self,
            visited: HashSet::default(),
            created: HashSet::default(),
            archived: HashSet::default(),
        };
        choice.update().accept(&mut extractor);
        if choice.consuming() {
            let template_name = DamlTyCon::new_absolute(package_id, module_path, template_name).to_static();
            extractor.archived.insert(template_name);
        }
        ChoiceEvents::new(extractor.created, extractor.archived)
    }
}

// TODO unwrap here
struct ChoiceEventVisitor<'arc> {
    archive: &'arc DamlArchive<'arc>,
    visited: HashSet<String>,
    created: HashSet<DamlTyCon<'arc>>,
    archived: HashSet<DamlTyCon<'arc>>,
}

impl DamlElementVisitor for ChoiceEventVisitor<'_> {
    fn pre_visit_value_name<'a>(&mut self, value_name: &'a DamlValueName<'a>) {
        let name = value_name.to_string();
        if !self.visited.contains(&name) {
            self.visited.insert(name);
            self.archive.value_by_name(value_name).unwrap().accept(self);
        }
    }

    fn pre_visit_create<'a>(&mut self, create: &DamlCreate<'a>) {
        let template_name = DamlTyCon::new(Box::new(create.template().to_static()), vec![]);
        self.created.insert(template_name);
    }

    fn pre_visit_exercise<'a>(&mut self, exercise: &DamlExercise<'a>) {
        if exercise.choice() == ARCHIVE_CHOICE_NAME {
            let template_name = DamlTyCon::new(Box::new(exercise.template().to_static()), vec![]);
            self.archived.insert(template_name);
        } else {
            let data = self.archive.data_by_tycon_name(exercise.template()).unwrap();
            if let DamlData::Template(template) = data {
                let choice = template.choices().iter().find(|c| c.name() == exercise.choice()).unwrap();
                if choice.consuming() {
                    let template_name = DamlTyCon::new(Box::new(exercise.template().to_static()), vec![]);
                    self.archived.insert(template_name);
                }
            }
            data.accept(self);
        }
    }

    fn pre_visit_exercise_by_key<'a>(&mut self, exercise_by_key: &DamlExerciseByKey<'a>) {
        self.archive.data_by_tycon_name(exercise_by_key.template()).unwrap().accept(self);
    }
}
