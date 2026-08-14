#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationMenuItem {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub page_id: Option<String>,
    pub enabled: bool,
    pub children: Vec<ApplicationMenuItem>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationUser {
    pub label: String,
    pub handle: String,
    pub initials: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationAccountAction {
    AgentSettings,
    Profile,
    ChangePassword,
    SignOut,
}
