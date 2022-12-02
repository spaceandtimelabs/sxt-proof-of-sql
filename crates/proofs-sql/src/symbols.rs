use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;

/// Case-insensitive name of a table/column.
///
/// Names are case-insensitive for the purpose of comparison since they usually are in SQL.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Name {
    /// The name itself which is always in lower case
    string: String,
}

impl Name {
    pub fn new(string: String) -> Name {
        Name {
            string: string.to_lowercase(),
        }
    }

    pub fn as_str(&self) -> &str {
        self.string.as_str()
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.string.fmt(f)
    }
}

impl<'a> From<&'a str> for Name {
    fn from(value: &'a str) -> Name {
        Name::new(String::from(value))
    }
}

impl From<String> for Name {
    fn from(value: String) -> Name {
        Name::new(value)
    }
}

impl PartialEq<str> for Name {
    fn eq(&self, other: &str) -> bool {
        self.string.eq(&other.to_lowercase())
    }
}

impl PartialOrd<str> for Name {
    fn partial_cmp(&self, other: &str) -> Option<Ordering> {
        self.string.partial_cmp(&other.to_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strings_are_lower_case_when_converted_to_names() {
        let raw_str = "sxt";
        let string = "sXt".to_owned();
        let lower_case = Name::from(raw_str);
        let upper_case = Name::new("SXT".to_owned());
        let mixed_case = Name::from(string);
        // Everything is set to lower case
        assert_eq!(lower_case, upper_case);
        assert_eq!(lower_case, mixed_case);
        assert_eq!(lower_case.as_str(), "sxt");
    }
}
