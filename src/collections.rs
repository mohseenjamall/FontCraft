//! Filters over the whole library: what is turned on, fonts by writing system, variable fonts
//! and families with many weights. Exactly one filter (or one folder) is active at a time.
use crate::app::Family;
use crate::samples::{ARABIC, LATIN};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Collection {
    TurnedOn,
    Arabic,
    English,
    Variable,
    ManyWeights,
}

impl Collection {
    /// In sidebar order; `TurnedOn` is drawn in the Library group, the rest under Collections.
    pub const ALL: [Collection; 5] =
        [Collection::TurnedOn, Collection::Arabic, Collection::English, Collection::Variable, Collection::ManyWeights];

    #[allow(dead_code)] // drawn by the sidebar in the next task
    pub fn label(self) -> &'static str {
        match self {
            Collection::TurnedOn => "Turned on",
            Collection::Arabic => "Arabic",
            Collection::English => "English",
            Collection::Variable => "Variable",
            Collection::ManyWeights => "4+ weights",
        }
    }

    #[allow(dead_code)] // drawn by the sidebar in the next task
    pub fn hint(self) -> &'static str {
        match self {
            Collection::TurnedOn => "Fonts that are on for you, whoever turned them on",
            Collection::Arabic => "Fonts that include Arabic letters",
            Collection::English => "Fonts that include Latin letters",
            Collection::Variable => "Fonts with adjustable axes such as weight or width",
            Collection::ManyWeights => "Families with more than 3 different weights",
        }
    }

    /// Position in [`Collection::ALL`], used for the cached counts.
    pub fn index(self) -> usize {
        Self::ALL.iter().position(|c| *c == self).unwrap_or(0)
    }

    /// `on` is whether every file of the family is turned on (only `TurnedOn` cares).
    pub fn contains(self, fam: &Family, on: bool) -> bool {
        match self {
            Collection::TurnedOn => on,
            Collection::Arabic => fam.scripts & ARABIC != 0,
            Collection::English => fam.scripts & LATIN != 0,
            Collection::Variable => fam.variable,
            Collection::ManyWeights => fam.weight_count > 3,
        }
    }

    /// For the `--collection` screenshot flag: turnedon, arabic, english, variable, weights.
    pub fn parse(name: &str) -> Option<Collection> {
        match name.trim().to_lowercase().as_str() {
            "turnedon" => Some(Collection::TurnedOn),
            "arabic" => Some(Collection::Arabic),
            "english" => Some(Collection::English),
            "variable" => Some(Collection::Variable),
            "weights" => Some(Collection::ManyWeights),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scan::Face;
    use std::path::PathBuf;

    fn fam(scripts: u16, variable: bool, weights: usize) -> Family {
        let faces = (0..weights.max(1))
            .map(|i| Face {
                path: PathBuf::from(format!("f{i}.ttf")),
                index: 0,
                family: "F".into(),
                style: format!("S{i}"),
                weight: 100 + 100 * i as u16,
                italic: false,
                script: 0,
                scripts,
                variable: variable && i == 0,
                source: 0,
            })
            .collect();
        Family::new("F".into(), faces)
    }

    #[test]
    fn collection_membership() {
        assert!(Collection::Arabic.contains(&fam(ARABIC | LATIN, false, 1), false));
        assert!(Collection::English.contains(&fam(ARABIC | LATIN, false, 1), false));
        assert!(!Collection::Arabic.contains(&fam(LATIN, false, 1), false));
        assert!(!Collection::English.contains(&fam(ARABIC, false, 1), false));
        assert!(Collection::Variable.contains(&fam(0, true, 1), false));
        assert!(!Collection::Variable.contains(&fam(0, false, 1), false));
        assert!(!Collection::ManyWeights.contains(&fam(0, false, 3), false));
        assert!(Collection::ManyWeights.contains(&fam(0, false, 4), false));
        assert!(Collection::TurnedOn.contains(&fam(0, false, 1), true));
        assert!(!Collection::TurnedOn.contains(&fam(0, false, 1), false));
    }

    #[test]
    fn labels_and_parse_agree() {
        for c in Collection::ALL {
            assert_eq!(Collection::ALL[c.index()], c);
            assert!(!c.label().is_empty() && !c.hint().is_empty());
        }
        assert_eq!(Collection::parse(" TurnedOn "), Some(Collection::TurnedOn));
        assert_eq!(Collection::parse("weights"), Some(Collection::ManyWeights));
        assert_eq!(Collection::parse("nope"), None);
    }
}
