use arrayvec::ArrayString;

use crate::sql::IdentifierParser;
use crate::{ParseError, ParseResult};
use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

/// Top-level unique identifier.
#[derive(Debug, PartialEq, Eq, Clone, Hash, Ord, PartialOrd, Copy)]
pub struct Identifier {
    name: ArrayString<64>,
}

impl Identifier {
    /// Constructor for [Identifier]
    ///
    /// Note: this constructor should be private within the proofs_sql crate.
    /// This is necessary to guarantee that no one outside the crate
    /// can create Names, thus securing that ResourceIds and Identifiers
    /// are always valid postgresql identifiers.
    pub(crate) fn new<S: AsRef<str>>(string: S) -> Self {
        Self {
            name: ArrayString::from(&string.as_ref().to_lowercase()).expect("Identifier too long"),
        }
    }

    /// An alias for [Identifier::from_str], provided for convenience.
    pub fn try_new<S: AsRef<str>>(string: S) -> ParseResult<Self> {
        Self::from_str(string.as_ref())
    }

    /// The name of this [Identifier]
    /// It already implements [Deref] to [str], so this method is not necessary for most use cases.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// An alias for [Identifier::name], provided for convenience.
    pub fn as_str(&self) -> &str {
        self.name()
    }
}

impl FromStr for Identifier {
    type Err = ParseError;

    fn from_str(string: &str) -> ParseResult<Self> {
        let name = IdentifierParser::new()
            .parse(string)
            .map_err(|e| ParseError::IdentifierParseError(format!("{:?}", e)))?;

        Ok(Identifier::new(name))
    }
}
crate::impl_serde_from_str!(Identifier);

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.name.fmt(f)
    }
}

impl PartialEq<str> for Identifier {
    fn eq(&self, other: &str) -> bool {
        other.eq_ignore_ascii_case(&self.name)
    }
}

impl PartialOrd<str> for Identifier {
    fn partial_cmp(&self, other: &str) -> Option<Ordering> {
        self.name.partial_cmp(other.to_lowercase().as_str())
    }
}

impl std::ops::Deref for Identifier {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.name.as_str()
    }
}

impl AsRef<str> for Identifier {
    fn as_ref(&self) -> &str {
        self.name.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str_identifier() {
        assert_eq!(
            Identifier::from_str("GOOD_IDENTIFIER13AD_IDENTIFIER")
                .unwrap()
                .name(),
            "good_identifier13ad_identifier"
        );
    }

    #[test]
    fn from_str_identifier_with_additional_characters_fails() {
        assert!(Identifier::from_str("GOOD_IDENTIFIER.").is_err());
        assert!(Identifier::from_str("BAD$IDENTIFIER").is_err());
        assert!(Identifier::from_str("BAD IDENTIFIER").is_err());
    }

    #[test]
    fn identifier_from_str() {
        assert_eq!(
            Identifier::from_str("G00d_identifier").unwrap().name(),
            "g00d_identifier"
        );
    }

    #[test]
    fn display_identifier() {
        assert_eq!(
            Identifier::from_str("GOOD_IDENTIFIER").unwrap().to_string(),
            "good_identifier"
        );

        assert_eq!(
            Identifier::from_str("_can_Start_with_underscore")
                .unwrap()
                .to_string(),
            "_can_start_with_underscore"
        );
    }

    #[test]
    fn we_can_parse_valid_identifiers_with_white_spaces_at_beginning_or_end() {
        assert_eq!(
            Identifier::from_str(" GOOD_IDENTIFIER").unwrap().name(),
            "good_identifier"
        );
        assert_eq!(
            Identifier::from_str("GOOD_IDENTIFIER ").unwrap().name(),
            "good_identifier"
        );
    }

    #[test]
    fn we_cannot_parse_invalid_identifiers() {
        assert!(Identifier::from_str("").is_err());
        assert!(Identifier::from_str(".").is_err());
        assert!(Identifier::from_str("GOOD_IDENTIFIER:GOOD_IDENTIFIER").is_err());
        assert!(Identifier::from_str("BAD$IDENTIFIER").is_err());
        assert!(Identifier::from_str("BAD_IDENT!FIER").is_err());
        assert!(Identifier::from_str("BAD IDENTIFIER").is_err());
        assert!(Identifier::from_str("13AD_IDENTIFIER").is_err());
        assert!(Identifier::from_str("$AD_IDENTIFIER").is_err());
        assert!(Identifier::from_str("GOOD_IDENTIFIER.").is_err());
        assert!(Identifier::from_str(".GOOD_IDENTIFIER").is_err());
        assert!(Identifier::from_str(&"LONG_IDENTIFIER_OVER_64_CHARACTERS".repeat(12)).is_err());
    }

    #[test]
    fn serialize_works() {
        let identifier = Identifier::from_str("GOOD_IDENTIFIER").unwrap();
        let serialized = serde_json::to_string(&identifier).unwrap();
        assert_eq!(serialized, r#""good_identifier""#);
    }

    #[test]
    fn deserialize_works() {
        let identifier = Identifier::from_str("GOOD_IDENTIFIER").unwrap();
        let deserialized: Identifier = serde_json::from_str(r#""good_identifier""#).unwrap();
        assert_eq!(identifier, deserialized);
    }

    #[test]
    fn deserialize_fails_on_invalid_identifier() {
        let deserialized: Result<Identifier, _> = serde_json::from_str(r#""BAD IDENTIFIER""#);
        assert!(deserialized.is_err());
    }

    #[test]
    fn deserialize_fails_on_empty_string() {
        let deserialized: Result<Identifier, _> = serde_json::from_str(r#""""#);
        assert!(deserialized.is_err());
    }

    #[test]
    fn deserialize_fails_on_long_identifier() {
        let deserialized: Result<Identifier, _> = serde_json::from_str(&format!(
            r#""{}""#,
            "LONG_IDENTIFIER_OVER_64_CHARACTERS".repeat(12)
        ));
        assert!(deserialized.is_err());
    }

    #[test]
    fn deserialize_works_in_a_type_parameter() {
        let deserialized: Vec<Identifier> =
            serde_json::from_str(r#"[ "good_identifier" ]"#).unwrap();
        assert_eq!(
            deserialized,
            vec![Identifier::from_str("GOOD_IDENTIFIER").unwrap()]
        );
    }

    #[test]
    fn strings_are_lower_case_when_converted_to_names() {
        let raw_str = "sxt";
        let string = "sXt".to_owned();
        let lower_case = Identifier::new(raw_str);
        let upper_case = Identifier::new("SXT");
        let mixed_case = Identifier::new(string);
        // Everything is set to lower case
        assert_eq!(lower_case, upper_case);
        assert_eq!(lower_case, mixed_case);
        assert_eq!(lower_case.name(), "sxt");
    }

    #[test]
    #[should_panic]
    fn long_names_panic() {
        Identifier::new("t".repeat(65));
    }

    #[test]
    #[should_panic]
    fn long_unicode_names_panic() {
        Identifier::new("茶".repeat(22));
    }

    #[test]
    fn short_names_are_fine() {
        Identifier::new("t".repeat(64));
        Identifier::new("茶".repeat(21));
    }
}
