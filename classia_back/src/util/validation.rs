use validator::ValidationErrors;

pub fn validation_message(errors: ValidationErrors) -> String {
    errors
        .field_errors()
        .iter()
        .flat_map(|(field, field_errors)| {
            field_errors.iter().map(move |error| {
                let msg = error.message.as_deref().unwrap_or("Dato inválido");
                format!("{field}: {msg}")
            })
        })
        .collect::<Vec<_>>()
        .join(", ")
}