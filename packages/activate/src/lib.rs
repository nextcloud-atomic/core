use std::env;
use std::fmt::Display;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use axum::extract::FromRef;
use secrets::SecretVec;
use tera::Context;
use errors::NcAtomicError;
use core::NCATOMIC_VERSION;
use core::config::{NcaConfig, NcAioConfig};
use kvp::KeyValueProvider;
use core::templating::render_template;
use core::crypto::{LockableSecret};

pub mod ui;

trait ValidatableConfig {
    fn validate(self) -> Result<(), NcAtomicError>;
}

#[derive(Clone, Debug)]
struct DiskConfig {
    name: String,
    format: Option<String>,
    encrypt: bool,
    mount_point: String,
}

impl ValidatableConfig for DiskConfig {
    fn validate(self) -> Result<(), NcAtomicError> {
        todo!()
    }
}

const PASSWORD_MINLENGTH_WEB: usize = 12;
const PASSWORD_MINLENGTH_OFFLINE: usize = 24;


#[derive(Debug, Clone, Eq, PartialEq)]
pub struct WebPassword (String);
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OfflinePassword (String);

impl ValidatableConfig for WebPassword {
    fn validate(self) -> Result<(), NcAtomicError> {
        if self.0.len() < PASSWORD_MINLENGTH_WEB {
            return Err(NcAtomicError::WeakPassword(PASSWORD_MINLENGTH_WEB, self.0.len()))
        }
        Ok(())
    }
}
impl ValidatableConfig for OfflinePassword {
    fn validate(self) -> Result<(), NcAtomicError> {
        if self.0.len() < PASSWORD_MINLENGTH_OFFLINE {
            return Err(NcAtomicError::WeakPassword(PASSWORD_MINLENGTH_OFFLINE, self.0.len()))
        }
        Ok(())
    }
}
impl Display for WebPassword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.clone())
    }
}
impl Display for OfflinePassword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.clone())
    }
}

#[derive(Clone, Debug)]
struct CredentialsConfig {
    master_password: OfflinePassword,
    admin_password: WebPassword,
    nextcloud_admin_password: WebPassword,
    backup_password: OfflinePassword,
}

impl ValidatableConfig for CredentialsConfig {
    fn validate(self) -> Result<(), NcAtomicError> {
        self.master_password.validate()?;
        self.nextcloud_admin_password.validate()?;
        self.backup_password.validate()?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct ActivationConfig {
    credentials: Option<CredentialsConfig>,
    disk_config: Option<Vec<DiskConfig>>,
    nc_storage_directory: String,
}

pub struct ActivationContext {
    config_template_directory: String,
    config_render_target_directory: String,
}

impl ActivationContext {
    pub fn from_env() -> Result<ActivationContext, NcAtomicError> {
        Ok(ActivationContext{
            config_template_directory: env::var("NCA_CONFIG_SOURCE")
                .map_err(NcAtomicError::new_server_config_error)?,
            config_render_target_directory: env::var("NCA_CONFIG_TARGET")
                .map_err(NcAtomicError::new_server_config_error)?
        })
    }
}

impl ValidatableConfig for ActivationConfig {
    fn validate(self) -> Result<(), NcAtomicError> {
        self.credentials
            .ok_or(NcAtomicError::MissingConfig("Missing configuration for passwords".to_string()))?
            .validate()?;
        if let Some(disks) = self.disk_config {
            disks.iter().try_for_each(|d| d.clone().validate())?;
        }
        let nc_storage_path = Path::new(self.nc_storage_directory.as_str());
        if nc_storage_path.is_relative() {
            Err(NcAtomicError::InvalidPath(self.nc_storage_directory, "Must be absolute".to_string()))
        } else if nc_storage_path.is_file() {
            Err(NcAtomicError::InvalidPath(self.nc_storage_directory, "Exists and is a file".to_string()))
        } else if nc_storage_path.is_dir() {
            match nc_storage_path.read_dir() {
                Err(e) => Err(NcAtomicError::Unexpected(e.to_string())),
                Ok(mut files) => {
                    match files.any(|_| true) { 
                        true => Err(NcAtomicError::InvalidPath(self.nc_storage_directory, "Exists and is not empty.".to_string())),
                        false => Ok(())
                    }
                }
            }
        } else {
            Ok(())
        }
    }
}

fn render_aio_config(cfg: NcAioConfig, aio_template_path: PathBuf, aio_render_path: PathBuf) -> Result<(), NcAtomicError> {
    let mut tera_ctx = Context::new();
    tera_ctx.insert("NC_AIO_CONFIG", &cfg.to_map());
    render_template(tera_ctx.clone(),
                    aio_template_path.join("defaults.env.j2"),
                    aio_render_path.join(".env"))
        .map_err(NcAtomicError::new_io_error)?;
    render_template(tera_ctx,
                    aio_template_path.join("compose.yaml.j2"),
                    aio_render_path.join("compose.yaml"))
        .map_err(NcAtomicError::new_io_error)?;
    Ok(())
}

pub fn activate(ctx: ActivationContext, config: ActivationConfig) -> Result<(), NcAtomicError> {
    config.clone().validate()?;
    let nc_storage_path = Path::new(config.nc_storage_directory.as_str());
    create_dir_all(nc_storage_path)
        .map_err(|e| NcAtomicError::InvalidPath(config.nc_storage_directory, e.to_string()))?;

    let mut cfg = NcaConfig::new(NCATOMIC_VERSION, None).map_err(NcAtomicError::new_unexpected_error)?;
    let mut pw_bytes = config.credentials.as_ref().unwrap().master_password.to_string();
    let password: SecretVec<u8> = SecretVec::new(pw_bytes.len(), |s| {
        s.copy_from_slice(pw_bytes.as_bytes());
    });
    let masterkey = cfg.get_masterkey(password);
    cfg.unlock(&masterkey).map_err(NcAtomicError::new_crypto_error)?;
    let mut admin_pw_str = config.credentials.unwrap().admin_password.to_string();
    let admin_pw_val: SecretVec<u8> = SecretVec::new(admin_pw_str.len(), |s| {
        s.copy_from_slice(admin_pw_str.as_bytes());
    });
    cfg.admin_password = LockableSecret::new_encrypted(&masterkey, admin_pw_val, cfg.get_salt())
        .map_err(NcAtomicError::new_crypto_error)?;

    let config_template_base_path = PathBuf::from(ctx.config_template_directory);
    let config_render_base_path = PathBuf::from(ctx.config_render_target_directory);
    cfg.save(config_render_base_path.join("ncatomic.json"))
        .map_err(NcAtomicError::new_io_error)?;
    render_aio_config(cfg.nc_aio,
                      config_template_base_path.join("nextcloud-aio"),
                      config_render_base_path.join("nextcloud-aio"))?;
    Ok(())
}

pub fn unlock(ctx: ActivationContext, password: OfflinePassword) -> Result<(), NcAtomicError> {
    let config_template_base_path = PathBuf::from(ctx.config_template_directory);
    let config_render_base_path = PathBuf::from(ctx.config_render_target_directory);
    let config_path = config_render_base_path.join("ncatomic.json");
    if !config_path.exists() {
        return Err(NcAtomicError::ServerConfiguration("Config file does not exist".to_string()));
    }
    let mut cfg = NcaConfig::load(config_path)
        .map_err(NcAtomicError::new_server_config_error)?;

    let pw_str = password.to_string();
    let password: SecretVec<u8> = SecretVec::new(pw_str.len(), |s| {
        s.copy_from_slice(pw_str.as_bytes());
    });
    let masterkey = cfg.get_masterkey(password);
    cfg.unlock(&masterkey).map_err(NcAtomicError::new_crypto_error)?;

    render_aio_config(cfg.nc_aio,
                      config_template_base_path.join("nextcloud-aio"),
                      config_render_base_path.join("nextcloud-aio"))?;
    Ok(())
}

pub fn is_activated(ctx: ActivationContext) -> Result<bool, NcAtomicError> {
    let config_render_base_path = PathBuf::from(ctx.config_render_target_directory);
    let _ = NcaConfig::load(config_render_base_path.join("ncatomic.json"))
        .map_err(NcAtomicError::new_server_config_error)?;
    Ok(true)
}


#[cfg(test)]
mod tests {
    use std::fs;
    use std::fs::File;
    use std::io::{Read, Write};
    use secrets::{Secret, SecretBox, SecretVec};
    use super::*;
    use tempdir::TempDir;
    use core::crypto::{encode, LockableSecret, secret_to_secret_string};

    #[test]
    fn activation_generates_config_files() {
        let tmp_dir = TempDir::new("ncatest").expect("failed to create tmp dir");
        // let template_dir = tmp_dir.path().join("templates");
        let render_dir = tmp_dir.path().join("target");
        create_dir_all(&render_dir).expect("failed to create render dir");
        let nc_storage_dir = tmp_dir.path().join("ncdata");

        let template_dir = env::current_exe().unwrap().parent().unwrap().join("../../../resource/templates/");
        
        let ctx = ActivationContext{
            config_template_directory: template_dir.to_string_lossy().to_string(),
            config_render_target_directory: render_dir.to_string_lossy().to_string(),
        };
        let config = ActivationConfig{
            disk_config: None,
            credentials: Some(CredentialsConfig{
                admin_password: WebPassword("very_secret_pw".to_string()),
                backup_password: OfflinePassword("abcdefghijklmnopqrstuvwxyz".to_string()),
                master_password: OfflinePassword("abcdefghijklmnopqrstuvwxyz".to_string()),
                nextcloud_admin_password: WebPassword("0123456789ab".to_string()),
            }),
            nc_storage_directory: nc_storage_dir.to_string_lossy().to_string(),
        };

        activate(ctx, config).expect("failed to activate");

        let mut cfg_new = NcaConfig::load(render_dir.join("ncatomic.json")).unwrap();
        assert_eq!(NCATOMIC_VERSION, cfg_new.ncatomic_version);

        let pw = "abcdefghijklmnopqrstuvwxyz";
        let pw: SecretVec<u8> = SecretVec::new(pw.len(), |s| {
            s.copy_from_slice(pw.as_bytes());
        });
        let masterkey = cfg_new.get_masterkey(pw);
        cfg_new.unlock(&masterkey).unwrap();

        let mut nc_aio_env = String::new();
        File::open(render_dir.join("nextcloud-aio/.env")).unwrap().read_to_string(&mut nc_aio_env).unwrap();

        let db_password = LockableSecret::new_derived(&masterkey, "AIO_DATABASE_PASSWORD".to_string(), cfg_new.get_salt()).unwrap();
        let db_pw_string = secret_to_secret_string(&db_password);
        assert!(nc_aio_env.contains(db_pw_string.as_str()));

        render_dir.read_dir().unwrap().for_each(|entry| {
            println!("{}", entry.unwrap().path().display());
        });

        let mut ncatomic_json = String::new();
        File::open(render_dir.join("ncatomic.json")).unwrap().read_to_string(&mut ncatomic_json).unwrap();
        println!("{}", ncatomic_json);

    }
    
    #[test]
    fn test_unlock_generates_config_files() {
        let cfg_json = stringify!({"nc_aio":{"db_password":{"locked":{"DERIVED":{"secret_id":"AIO_DATABASE_PASSWORD"}}},"fulltextsearch_pw":{"locked":{"DERIVED":{"secret_id":"AIO_FULLTEXTSEARCH_PASSWORD"}}},"nc_password":{"locked":"EMPTY"},"onlyoffice_secret":{"locked":{"DERIVED":{"secret_id":"AIO_ONLYOFFICE_SECRET"}}},"recording_secret":{"locked":{"DERIVED":{"secret_id":"AIO_RECORDING_SECRET"}}},"redis_password":{"locked":{"DERIVED":{"secret_id":"AIO_REDIS_PASSWORD"}}},"signaling_secret":{"locked":{"DERIVED":{"secret_id":"AIO_SIGNALING_SECRET"}}},"talk_internal_secret":{"locked":{"DERIVED":{"secret_id":"AIO_TALK_INTERNAL_SECRET"}}},"turn_secret":{"locked":{"DERIVED":{"secret_id":"AIO_TURN_SECRET"}}},"nc_domain":"nextcloudatomic.local","onlyoffice_enabled":false,"collabora_enabled":false,"talk_enabled":false,"talk_recording_enabled":false,"fulltextsearch_enabled":false,"clamav_enabled":false,"imaginary_enabled":false},"ncatomic_version":"0.1.0","admin_password":{"locked":{"ENCRYPTED":{"data":"VEBJ2TSNOFYSRXYCKB7L2HCVKEXXWKFZB3ZMUMV3PV5WVNGD","nonce":"74K227VPW3LPKOIPV5ZQ====","source_version":"0.1.0"}}},"salt":"K5LYT4LPH3O6ZVQMM2FWNPGO7M======"});

        let tmp_dir = TempDir::new("ncatest").expect("failed to create tmp dir");
        // let template_dir = tmp_dir.path().join("templates");
        let render_dir = tmp_dir.path().join("target");
        create_dir_all(&render_dir).expect("failed to create render dir");
        let nc_storage_dir = tmp_dir.path().join("ncdata");
        let template_dir = env::current_exe().unwrap().parent().unwrap().join("../../../resource/templates/");

        let pw = "abcdefghijklmnopqrstuvwxyz";
        let pw_arg = OfflinePassword(pw.to_string());
        let pw: SecretVec<u8> = SecretVec::new(pw.len(), |s| {
            s.copy_from_slice(pw.as_bytes());
        });

        fs::write(render_dir.join("ncatomic.json"), cfg_json).unwrap();
        // File::open(render_dir.join("ncatomic.json")).unwrap().write_all(cfg_json.as_bytes()).unwrap();
        
        let mut cfg = NcaConfig::load(render_dir.join("ncatomic.json")).unwrap();
        let masterkey = cfg.get_masterkey(pw.clone());
        cfg.unlock(&masterkey).unwrap();

        let ctx = ActivationContext{
            config_template_directory: template_dir.to_string_lossy().to_string(),
            config_render_target_directory: render_dir.to_string_lossy().to_string(),
        };
        
        unlock(ctx, pw_arg).unwrap();

        let mut cfg_new = NcaConfig::load(render_dir.join("ncatomic.json")).unwrap();
        assert_eq!(cfg.ncatomic_version, cfg_new.ncatomic_version);
        
        let masterkey = cfg_new.get_masterkey(pw);
        cfg_new.unlock(&masterkey).unwrap();

        let mut nc_aio_env = String::new();
        File::open(render_dir.join("nextcloud-aio/.env")).unwrap().read_to_string(&mut nc_aio_env).unwrap();

        let db_password = LockableSecret::new_derived(&masterkey, "AIO_DATABASE_PASSWORD".to_string(), cfg_new.get_salt()).unwrap();
        let db_pw_string = secret_to_secret_string(&db_password);
        assert!(nc_aio_env.contains(db_pw_string.as_str()));

        render_dir.read_dir().unwrap().for_each(|entry| {
            println!("{}", entry.unwrap().path().display());
        });

        let mut ncatomic_json = String::new();
        File::open(render_dir.join("ncatomic.json")).unwrap().read_to_string(&mut ncatomic_json).unwrap();
        println!("{}", ncatomic_json);
        
    }
}
