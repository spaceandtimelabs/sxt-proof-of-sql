use crate::sql::IdentifierParser;
use crate::symbols::Name;
use crate::{ParseError, ParseResult};
use std::fmt::{self, Display};
use std::str::FromStr;

/// Top-level unique identifier.
#[derive(Debug, PartialEq, Eq, Clone, Hash, Copy)]
pub struct Identifier {
    name: Name,
}

impl Identifier {
    /// Constructor for [Identifier]s.
    ///
    /// Note: this is a safe constructor, since
    /// only the proofs_sql parser can construct
    /// or modify `Name` objects (its constructor
    /// is using pub(crate)).
    pub fn new(name: Name) -> Self {
        Self { name }
    }

    /// Constructor for [Identifier]s.
    ///
    /// # Errors
    ///
    ///  Fails if the provided name string isn't valid a postgres-style
    ///  identifier (excluding dollar signs).
    pub fn try_new(name: &str) -> ParseResult<Self> {
        let name = IdentifierParser::new()
            .parse(name)
            .map_err(|e| ParseError::IdentifierParseError(format!("{:?}", e)))?;

        Ok(Self { name })
    }

    /// The name of this [Identifier].
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

impl FromStr for Identifier {
    type Err = ParseError;

    fn from_str(string: &str) -> ParseResult<Self> {
        let name = IdentifierParser::new()
            .parse(string)
            .map_err(|e| ParseError::IdentifierParseError(format!("{:?}", e)))?;

        Ok(Identifier { name })
    }
}

impl Display for Identifier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Identifier { name } = self;

        formatter.write_str(format!("{name}").as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_new_identifier() {
        assert_eq!(
            Identifier::try_new("GOOD_IDENTIFIER13AD_IDENTIFIER")
                .unwrap()
                .name(),
            "good_identifier13ad_identifier"
        );
    }

    #[test]
    fn try_new_identifier_with_additional_characters_fails() {
        assert!(Identifier::try_new("GOOD_IDENTIFIER.").is_err());
        assert!(Identifier::try_new("BAD$IDENTIFIER").is_err());
        assert!(Identifier::try_new("BAD IDENTIFIER").is_err());
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
            Identifier::try_new("GOOD_IDENTIFIER").unwrap().to_string(),
            "good_identifier"
        );

        assert_eq!(
            Identifier::try_new("_can_Start_with_underscore")
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
}
