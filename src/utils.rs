pub fn format_humanize<T>(count: T, singular: &str, plural: &str) -> String
where
    T: std::fmt::Display,
{
    if count.to_string() == "1" {
        format!("{} {}", count, singular)
    } else {
        format!("{} {}", count, plural)
    }
}