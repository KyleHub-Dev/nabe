#[derive(Debug, Clone)]
pub struct SubjectInput {
    pub id: String,
    pub provider: String,
    pub subject: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EdgeNodeInput {
    pub id: String,
    pub name: String,
    pub hostname: String,
    pub architecture: String,
    pub os: String,
    pub kernel: String,
    pub speiche_version: String,
    pub health_status: String,
    pub inventory_json: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EdgeNodeRecord {
    pub id: String,
    pub name: String,
    pub hostname: String,
    pub architecture: String,
    pub os: String,
    pub kernel: String,
    pub speiche_version: String,
    pub health_status: String,
    pub inventory_json: Option<String>,
    pub enrolled_at: String,
    pub last_seen_at: String,
}
