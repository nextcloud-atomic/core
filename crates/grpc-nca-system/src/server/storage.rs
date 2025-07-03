use rsblkid::cache::Cache;
use rsblkid::device::Tag;
use nca_error::NcaError;
use nca_system_api::systemd::api::set_fallback_disk_encryption_password;

pub(super) fn get_crypto_devices() -> Result<Vec<String>, NcaError> {

    let mut cache = Cache::builder()
        .discard_changes_on_drop()
        .build()
        .map_err(|e| NcaError::new_io_error(format!("Failed create blkid cache: {e:?}")))?;
    cache.probe_all_devices()
        .map_err(|e| NcaError::new_io_error(format!("Failed to scan for devices: {e:?}")))?;
    let crypt_tag: Tag = "TYPE=crypto_LUKS".parse()
        .map_err(|e| NcaError::new_io_error(format!("Failed create tag: {e:?}")))?;
    let swap_tag: Tag = "PARTLABEL=swap".parse()
        .map_err(|e| NcaError::new_io_error(format!("Failed create tag: {e:?}")))?;
    let crypt_disks = cache.iter()
        .filter(|d| d.has_tag(crypt_tag.clone()) && !d.has_tag(swap_tag.clone()));
    Ok(crypt_disks.filter_map(|d| d.name().to_path_buf().to_str().map(|s| s.to_string())).collect())

}


pub(super) async fn add_fallback_password_to_encrypted_disks(password: String) -> Result<(), NcaError> {

    let paths = get_crypto_devices()?;
    println!("paths of crypto devices: {:?}", paths);
    for device_path in paths {
        set_fallback_disk_encryption_password(password.clone(), device_path).await?;
    }
    Ok(())
}