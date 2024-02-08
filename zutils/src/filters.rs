use anyhow::Result;
use regex::Regex;
// use std::os::unix::process::CommandExt as _;

/// A generic structure for representing filters. The rules are:
///
///  * An empty set of filters means "pass everything"
///  * Anything else means "pass anything any filter matches"
///  * Each filter is a regex which must match the whole string to match.
///
pub struct FilterSet {
    pub filters: Vec<String>,
    pub res: Vec<Regex>,
}

impl FilterSet {
    pub fn new(filters: &Vec<String>) -> Result<Self> {
        // Do this with a loop so we can report errors ..
        let mut res: Vec<Regex> = Vec::new();
        for f in filters {
            let re = Regex::new(f)?;
            res.push(re);
        }
        Ok(Self {
            filters: filters.clone(),
            res,
        })
    }

    pub fn is_match(&self, cand: &str) -> bool {
        if self.filters.is_empty() {
            true
        } else {
            let matches = self
                .res
                .iter()
                .filter(|x| {
                    if let Some(cap) = x.captures(cand) {
                        &cap[0] == cand
                    } else {
                        false
                    }
                })
                .count();
            matches > 0
        }
    }
}
