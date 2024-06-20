static FORMAT: &[time::format_description::FormatItem] = format_description::parse(
    "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:9]"
).expect("Invalid format description");

fn parse_timestamp_with_nanoseconds(value: &str) -> Result<OffsetDateTime, time::Error> {
    OffsetDateTime::parse(value, &FORMAT)
}