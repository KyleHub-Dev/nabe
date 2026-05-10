#[derive(Debug, Clone)]
pub struct SubjectInput {
    pub id: String,
    pub provider: String,
    pub subject: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
}
