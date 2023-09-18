pub enum Error {
    FailedRelatingData(String, String),
    InconsistentRelatedData(String, String),
    NotFound(String),
    AlreadyExists
}