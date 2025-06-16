use serde::{Deserialize, Serialize};


pub mod setup {
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Clone, Debug)]
    pub struct CredentialsConfig {
        pub salt: String,
        pub primary_password: String,
        // pub mfa_backup_codes: [String; 16],
        pub disk_encryption_password: String,
        pub backup_password: String,
    }

    // #[derive(Deserialize, Serialize, Clone, Debug)]
    // pub struct MfaInitRequest {
    //     pub primary_password: String,
    // }
    
    #[derive(Deserialize, Serialize, Clone, Debug)]
    pub struct CredentialsInitRequest {
        pub primary_password: String,
        pub nextcloud_admin_password: String,
    }

    #[derive(Deserialize, Serialize, Clone, Debug)]
    pub struct ServicesConfig {
        pub admin_domain: String,
        pub nextcloud_domain: String,
        pub nextcloud_password: String,
    }


    #[derive(Deserialize, Serialize, Clone, Debug)]
    pub struct NcAtomicInitializationConfig {
        pub services: ServicesConfig,
        pub primary_password: String
    }
}
