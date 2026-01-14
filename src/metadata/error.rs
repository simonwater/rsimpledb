use thiserror::Error;

#[derive(Error, Debug)]
pub enum MetadataError {
    #[error("Table `{0}` not found")]
    TableNotFound(String),
    #[error("Field `{0}` not found")]
    FieldNotFound(String),
}
