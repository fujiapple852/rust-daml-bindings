use anyhow::Result;

use daml::lf::element::{
    DamlArchive, DamlChoice, DamlElementVisitor, DamlEnum, DamlModule, DamlPackage, DamlRecord, DamlTemplate,
    DamlVariant, DamlVisitableElement,
};
use daml::lf::DarFile;
use ptree::print_config::UTF_CHARS_BOLD;
use ptree::{print_tree_with, Color, PrintConfig, Style, TreeBuilder};

pub(crate) fn outline(dar_path: &str, package_opt: Option<&str>, module_opt: Option<&str>) -> Result<()> {
    struct ModuleTreeVisitor {
        tree: TreeBuilder,
    }

    impl DamlElementVisitor for ModuleTreeVisitor {
        fn sort_elements(&self) -> bool {
            true
        }

        fn pre_visit_archive<'a>(&mut self, archive: &'a DamlArchive<'a>) {
            self.tree.begin_child(archive.name.to_owned());
        }

        fn post_visit_archive<'a>(&mut self, _: &'a DamlArchive<'a>) {
            self.tree.end_child();
        }

        fn pre_visit_package<'a>(&mut self, package: &'a DamlPackage<'a>) {
            // self.tree.begin_child(format!("{} {:?} {}", package.name, package.version, package.package_id));
            self.tree.begin_child(package.name.to_string());
        }

        fn post_visit_package<'a>(&mut self, _: &'a DamlPackage<'a>) {
            self.tree.end_child();
        }

        fn pre_visit_module<'a>(&mut self, module: &'a DamlModule<'a>) {
            if !module.is_root() {
                self.tree.begin_child(module.name().to_string());
            }
        }

        fn post_visit_module<'a>(&mut self, module: &'a DamlModule<'a>) {
            if !module.is_root() {
                self.tree.end_child();
            }
        }

        fn pre_visit_template<'a>(&mut self, template: &'a DamlTemplate<'a>) {
            self.tree.begin_child(template.name.to_string());
        }

        fn post_visit_template<'a>(&mut self, _: &'a DamlTemplate<'a>) {
            self.tree.end_child();
        }

        fn pre_visit_choice<'a>(&mut self, choice: &'a DamlChoice<'a>) {
            self.tree.begin_child(choice.name.to_string());
        }

        fn post_visit_choice<'a>(&mut self, _: &'a DamlChoice<'a>) {
            self.tree.end_child();
        }

        fn pre_visit_record<'a>(&mut self, record: &'a DamlRecord<'a>) {
            self.tree.begin_child(record.name.to_string());
        }

        fn post_visit_record<'a>(&mut self, _: &'a DamlRecord<'a>) {
            self.tree.end_child();
        }

        fn pre_visit_variant<'a>(&mut self, variant: &'a DamlVariant<'a>) {
            self.tree.begin_child(variant.name.to_string());
        }

        fn post_visit_variant<'a>(&mut self, _: &'a DamlVariant<'a>) {
            self.tree.end_child();
        }

        fn pre_visit_enum<'a>(&mut self, data_enum: &'a DamlEnum<'a>) {
            self.tree.begin_child(data_enum.name.to_string());
        }

        fn post_visit_enum<'a>(&mut self, _: &'a DamlEnum<'a>) {
            self.tree.end_child();
        }
    }

    let dar = DarFile::from_file(dar_path)?;

    let mut visitor = ModuleTreeVisitor {
        tree: TreeBuilder::new("root".to_owned()),
    };

    dar.apply(|archive| match (package_opt, module_opt) {
        (Some(search_package), Some(search_module)) => archive
            .packages
            .get(search_package)
            .and_then(|p| p.root_module.child_module_path(&search_module.split('.').collect::<Vec<_>>()))
            .iter()
            .for_each(|m| m.accept(&mut visitor)),
        (Some(search_package), None) =>
            archive.packages.get(search_package).iter().for_each(|&p| p.accept(&mut visitor)),
        (None, _) => archive.accept(&mut visitor),
    })?;

    let tree = visitor.tree.build();

    let config = {
        let mut config = PrintConfig::from_env();
        config.branch = Style {
            foreground: Some(Color::Yellow),
            dimmed: true,
            ..Style::default()
        };
        config.leaf = Style {
            bold: true,
            ..Style::default()
        };
        config.characters = UTF_CHARS_BOLD.into();
        config.indent = 4;
        config
    };

    print_tree_with(&tree, &config)?;
    Ok(())
}
