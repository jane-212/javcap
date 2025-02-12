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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::path::PathBuf;
    use test_case::test_case;

    #[test_case("/home/user/.config", true; "absolute")]
    #[test_case(".config", false; "relative")]
    fn test_absolute_path(path: &str, is_absolute: bool) {
        let path = PathBuf::from(path);
        let actual = absolute_path(&path);
        assert_eq!(actual.is_ok(), is_absolute);
    }
}
