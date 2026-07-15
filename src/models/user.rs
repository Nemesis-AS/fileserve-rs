use sqlx::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type)]
#[sqlx(type_name = "TEXT")]
#[sqlx(rename_all = "lowercase")]
pub enum UserRole {
    ADMIN,
    USER,
}

#[derive(sqlx::FromRow)]
pub struct User {
    username: String,
    name: String,
    password: String,
    role: UserRole,
    avatar: Option<String>,
}
