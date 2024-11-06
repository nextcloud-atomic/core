use std::fs;
use std::fs::File;
use std::path::PathBuf;
use tera::{Context, Tera};
use anyhow::{anyhow, Result};

pub fn render_template(ctx: Context, path_in: PathBuf, path_out: PathBuf) -> Result<()> {
    fs::create_dir_all(path_out.parent()
        .ok_or(anyhow!("Invalid template output path: {path_out:?}"))?)?;
    let mut tera = Tera::default();
    tera.add_template_file(&path_in, Some("default")).map_err(|_| anyhow!("Failed to load template file {path_in:?}"))?;
    let out_file = File::options()
        .write(true)
        .create(true)
        .open(&path_out)?;
    tera.render_to("default", &ctx,  out_file).map_err(|e| anyhow!("Failed to render template to {path_out:?} ({e:?})"))?;
    //unsafe { libc::raise(libc::SIGTERM) }
    // exit(0);
    Ok(())
}
