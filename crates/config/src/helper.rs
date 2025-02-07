use std::path::Path;

use validator::ValidationError;

pub fn absolute_path(path: &Path) -> Result<(), ValidationError> {
    if !path.is_absolute() {
        let msg = format!("路径必须是绝对路径 > {}", path.display());
        let err = ValidationError::new("path").with_message(msg.into());
        return Err(err);
    }

    Ok(())
}
