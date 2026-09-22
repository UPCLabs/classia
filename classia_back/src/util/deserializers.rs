use serde::Deserialize;

fn trim_string(value: String) -> String {
    value.trim().to_owned()
}

pub fn deserialize_trimmed<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    String::deserialize(deserializer).map(trim_string)
}

pub fn deserialize_optional_trimmed<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer).map(|value| value.map(trim_string))
}
