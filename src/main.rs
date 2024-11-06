use color_eyre::Report;

mod init;

#[tracing::instrument]
fn main() -> Result<(), Report> {
    let args = init::initialize()?;
    let (src, dst) = (args.input, args.output);

    let dir_entries = walkdir::WalkDir::new(&src)
        .into_iter()
        .collect::<Result<Vec<_>, walkdir::Error>>()?;

    let src_paths = dir_entries
        .into_iter()
        .filter(|x| x.metadata().is_ok() && x.metadata().unwrap().is_file())
        .map(|x| x.into_path())
        .collect::<Vec<_>>();

    let dest_paths = src_paths
        .iter()
        .cloned()
        .map(|x| {
            x.strip_prefix(&src)
                .unwrap()
                .iter()
                .collect::<Vec<_>>()
                .join(std::ffi::OsStr::new("_"))
        })
        .map(|x| (&dst).join(x))
        .collect::<Vec<_>>();

    let zipped_src = src_paths.iter().zip(&dest_paths).collect::<Vec<_>>();

    match std::fs::create_dir_all(&dst) {
        Ok(_) => (),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => (),
        Err(e) => {
            tracing::error!("Failed to create destination directory: {}", e);
            return Err(e.into());
        }
    }

    for (source, dest) in zipped_src {
        tracing::trace!("Creating hard link from {:?} to {:?}", source, dest);
        if let Err(e) = std::fs::hard_link(source, dest) {
            tracing::error!(
                "Failed to create hard link from {:?} to {:?}: {}",
                source,
                dest,
                e
            );
            return Err(e.into());
        }
        tracing::trace!("Successfully created hard link");
    }

    Ok(())
}
