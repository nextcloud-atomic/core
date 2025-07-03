#[derive(Clone, Debug)]
pub struct CredentialsConfig {
    pub(crate) disk_encryption_password: String,
    pub(crate) backup_password: String,
}