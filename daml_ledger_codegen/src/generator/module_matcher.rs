use regex::Regex;

pub struct ModuleMatcher {
    matchers: Vec<Regex>,
}

impl ModuleMatcher {
    pub fn new(module_filter_regex: &[&str]) -> ModuleMatcher {
        let matchers = module_filter_regex
            .iter()
            .map(|&re| Regex::new(re))
            .collect::<Result<Vec<_>, _>>()
            .expect("invalid regex for module_filter_regex");
        ModuleMatcher {
            matchers,
        }
    }

    pub fn matches(&self, path: &str) -> bool {
        path.is_empty() || self.matchers.is_empty() || self.matchers.iter().any(|m| m.is_match(path))
    }
}
