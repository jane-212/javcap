use std::path::Path;

use validator::ValidationError;

pub fn absolute_path(path: &Path) -> Result<(), ValidationError> {
    if !path.is_absolute() {
        let msg = format!("should use absolute path: {}", path.display());
        let err = ValidationError::new("path").with_message(msg.into());
        return Err(err);
    }

    Ok(())
}
