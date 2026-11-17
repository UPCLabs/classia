//! Delivery files on the deliveries volume (DEC-03). Only the generated
//! `<uuid>.docx` name is stored in the database; the original name and the
//! directory are never exposed.

use std::path::Path;

use tracing::warn;
use uuid::Uuid;

use super::error::DeliveryError;

pub const MAX_FILE_BYTES: usize = 20 * 1024 * 1024;
/// Request body limit for the multipart routes: the file plus form overhead.
pub const MAX_REQUEST_BYTES: usize = 21 * 1024 * 1024;

const ZIP_SIGNATURE: &[u8] = b"PK\x03\x04";

pub struct UploadedFile {
    pub file_name: Option<String>,
    pub bytes: Vec<u8>,
}

pub fn validate(file: &UploadedFile) -> Result<(), DeliveryError> {
    if file.bytes.is_empty() {
        return Err(DeliveryError::Validation(
            "Debes adjuntar un archivo".into(),
        ));
    }
    if file.bytes.len() > MAX_FILE_BYTES {
        return Err(DeliveryError::FileTooLarge);
    }
    let has_docx_extension = file
        .file_name
        .as_deref()
        .is_some_and(|name| name.to_lowercase().ends_with(".docx"));
    if !has_docx_extension || !file.bytes.starts_with(ZIP_SIGNATURE) {
        return Err(DeliveryError::InvalidFileType);
    }
    Ok(())
}

/// Writes the file before the database transaction (SDS §9.5); the caller
/// removes it again if the transaction does not commit.
pub async fn save(dir: &Path, bytes: &[u8]) -> Result<String, DeliveryError> {
    let name = format!("{}.docx", Uuid::now_v7());
    let write = async {
        tokio::fs::create_dir_all(dir).await?;
        tokio::fs::write(dir.join(&name), bytes).await
    };
    write.await.map_err(|error| {
        tracing::error!("Could not store a delivery file: {error}");
        DeliveryError::FileUnavailable
    })?;
    Ok(name)
}

/// Best effort: a leftover file is an orphan (DEC-03), never a failed request.
pub async fn remove(dir: &Path, name: &str) {
    if let Err(error) = tokio::fs::remove_file(dir.join(name)).await {
        warn!("Could not remove delivery file {name}: {error}");
    }
}
