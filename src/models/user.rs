pub enum UserRole {
    ADMIN = "admin",
    USER = "user",
}

pub struct User {
    username: String,
    name: String,
    password: String,
    role: UserRole,
    avatar: Option<String>,
}
