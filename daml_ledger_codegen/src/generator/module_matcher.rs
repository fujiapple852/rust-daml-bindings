use crate::convert::error::{DamlCodeGenError, DamlCodeGenResult};
use regex::Regex;

pub struct ModuleMatcher {
    matchers: Vec<Regex>,
}

impl ModuleMatcher {
    pub fn new(module_filter_regex: &[&str]) -> DamlCodeGenResult<ModuleMatcher> {
        let matchers = module_filter_regex
            .iter()
            .map(|&re| Regex::new(re))
            .collect::<Result<Vec<_>, _>>()
            .map_err(DamlCodeGenError::InvalidModuleMatcherRegex)?;
        Ok(ModuleMatcher {
            matchers,
        })
    }

    pub fn matches(&self, path: &str) -> bool {
        path.is_empty() || self.matchers.is_empty() || self.matchers.iter().any(|m| m.is_match(path))
    }
}
