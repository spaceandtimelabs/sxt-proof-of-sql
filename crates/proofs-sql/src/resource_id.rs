use crate::sql::ResourceIdParser;
use crate::{Identifier, ParseError, ParseResult};
use std::{
    fmt::{self, Display},
    str::FromStr,
};

/// Unique resource identifier, like `schema.object_name`.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ResourceId {
    schema: Identifier,
    object_name: Identifier,
}

impl ResourceId {
    /// Constructor for [ResourceId]s.
    pub fn new(schema: Identifier, object_name: Identifier) -> Self {
        Self {
            schema,
            object_name,
        }
    }

    /// Constructor for [ResourceId]s.
    ///
    /// # Errors
    /// Fails if the provided schema/object_name strings aren't valid postgres-style
    /// identifiers (excluding dollar signs).
    /// These identifiers are defined here:
    /// <https://www.postgresql.org/docs/current/sql-syntax-lexical.html#SQL-SYNTAX-IDENTIFIERS>.
    pub fn try_new(schema: &str, object_name: &str) -> ParseResult<Self> {
        let schema = Identifier::try_new(schema)?;
        let object_name = Identifier::try_new(object_name)?;

        Ok(ResourceId {
            schema,
            object_name,
        })
    }

    /// The schema identifier of this [ResourceId].
    pub fn schema(&self) -> &Identifier {
        &self.schema
    }

    /// The object_name identifier of this [ResourceId].
    pub fn object_name(&self) -> &Identifier {
        &self.object_name
    }

    /// Conversion to string in the format used in KeyDB.
    ///
    /// Space and time APIs accept a `.` separator in resource ids.
    /// However, when a resource id is stored in KeyDB, or used as a key, a `:` separator is used.
    /// This method differs from [ResourceId::to_string] by using the latter format.
    ///
    /// Furthermore, while space and time APIs accept lowercase resource identifiers,
    /// all resource identifiers are stored internally in uppercase.
    /// This method performs that transformation as well.
    /// For more information, see
    /// <https://space-and-time.atlassian.net/wiki/spaces/SE/pages/4947974/Gateway+Storage+Overview#Database-Resources>.
    pub fn storage_format(&self) -> String {
        let ResourceId {
            schema,
            object_name,
        } = self;

        let schema = schema.name().to_string().to_uppercase();
        let object_name = object_name.name().to_string().to_uppercase();

        format!("{schema}:{object_name}")
    }
}

impl Display for ResourceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ResourceId {
            schema,
            object_name,
        } = self;

        formatter.write_str(format!("{schema}.{object_name}").as_str())
    }
}

impl FromStr for ResourceId {
    type Err = ParseError;

    fn from_str(string: &str) -> ParseResult<Self> {
        let (schema, object_name) = ResourceIdParser::new()
            .parse(string)
            .map_err(|e| ParseError::ResourceIdParseError(format!("{:?}", e)))?;

        // use unsafe `Identifier::new` to prevent double parsing the ids
        Ok(ResourceId {
            schema: Identifier::new(schema),
            object_name: Identifier::new(object_name),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_new_resource_id() {
        let resource_id =
            ResourceId::try_new("G00d_identifier", "_can_start_with_underscore").unwrap();
        assert_eq!(resource_id.schema().name(), "g00d_identifier");
        assert_eq!(
            resource_id.object_name().name(),
            "_can_start_with_underscore"
        );
    }

    #[test]
    fn resource_id_from_str() {
        let resource_id =
            ResourceId::from_str("G00d_identifier._can_start_with_underscore").unwrap();
        assert_eq!(resource_id.schema().name(), "g00d_identifier");
        assert_eq!(resource_id.schema().name(), "g00d_identifier");
    }

    #[test]
    fn try_new_resource_id_with_additional_characters_fails() {
        assert!(ResourceId::try_new("GOOD_IDENTIFIER", "GOOD_IDENTIFIER.").is_err());
        assert!(ResourceId::try_new("GOOD_IDENTIFIER.", "GOOD_IDENTIFIER").is_err());
        assert!(ResourceId::try_new("BAD$IDENTIFIER", "GOOD_IDENTIFIER").is_err());
        assert!(ResourceId::try_new("GOOD_IDENTIFIER", "BAD IDENTIFIER").is_err());
    }

    #[test]
    fn we_can_parse_valid_resource_ids_with_white_spaces_at_beginning_or_end() {
        let resource_id =
            ResourceId::from_str("      GOOD_IDENTIFIER._can_start_with_underscore   ").unwrap();
        assert_eq!(resource_id.schema().name(), "good_identifier");
        assert_eq!(
            resource_id.object_name().name(),
            "_can_start_with_underscore"
        );

        let resource_id = ResourceId::try_new(
            "      GOOD_IDENTIFIER     ",
            "      _can_start_with_underscore   ",
        )
        .unwrap();
        assert_eq!(resource_id.schema().name(), "good_identifier");
        assert_eq!(
            resource_id.object_name().name(),
            "_can_start_with_underscore"
        );
    }

    #[test]
    fn display_resource_id() {
        assert_eq!(
            ResourceId::try_new("GOOD_IDENTIFIER", "good_identifier")
                .unwrap()
                .to_string(),
            "good_identifier.good_identifier"
        );

        assert_eq!(
            ResourceId::try_new("g00d_identifier", "_can_Start_with_underscore")
                .unwrap()
                .to_string(),
            "g00d_identifier._can_start_with_underscore"
        );
    }

    #[test]
    fn resource_id_storage_format() {
        assert_eq!(
            ResourceId::try_new("GOOD_IDENTIFIER", "good_identifier")
                .unwrap()
                .storage_format(),
            "GOOD_IDENTIFIER:GOOD_IDENTIFIER"
        );
        assert_eq!(
            ResourceId::try_new("g00d_identifier", "_can_Start_with_underscore")
                .unwrap()
                .storage_format(),
            "G00D_IDENTIFIER:_CAN_START_WITH_UNDERSCORE"
        );
    }

    #[test]
    fn invalid_resource_id_parsing_fails() {
        assert!(ResourceId::from_str("GOOD_IDENTIFIER").is_err());
        assert!(ResourceId::from_str("GOOD_IDENTIFIER:GOOD_IDENTIFIER").is_err());
        assert!(ResourceId::from_str("BAD$IDENTIFIER.GOOD_IDENTIFIER").is_err());
        assert!(ResourceId::from_str("GOOD_IDENTIFIER.BAD_IDENT!FIER").is_err());
        assert!(ResourceId::from_str("GOOD_IDENTIFIER.BAD IDENTIFIER").is_err());
        assert!(ResourceId::from_str("GOOD_IDENTIFIER.13AD_IDENTIFIER").is_err());
        assert!(ResourceId::from_str("13AD_IDENTIFIER.GOOD_IDENTIFIER").is_err());
        assert!(ResourceId::from_str("GOOD_IDENTIFIER.").is_err());
        assert!(ResourceId::from_str(".GOOD_IDENTIFIER").is_err());
        assert!(ResourceId::from_str(".").is_err());
        assert!(ResourceId::from_str("GOOD_IDENTIFIER").is_err());
        assert!(ResourceId::from_str("GOOD_IDENTIFIER.GOOD_IDENTIFIER.GOOD_IDENTIFIER").is_err());
    }
}
