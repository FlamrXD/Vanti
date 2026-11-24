use notify::{event::CreateKind, Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc;

pub fn watch_added_files<P, F>(dir: P, recursive: bool, mut on_file_added: F) -> notify::Result<()>
where
    P: AsRef<Path>,
    F: FnMut(PathBuf),
{
    let (tx, rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    let mode = if recursive {
        RecursiveMode::Recursive
    } else {
        RecursiveMode::NonRecursive
    };

    watcher.watch(dir.as_ref(), mode)?;

    for res in rx {
        match res {
            Ok(Event {
                kind: EventKind::Create(CreateKind::File),
                paths,
                ..
            }) => {
                for path in paths {
                    on_file_added(path);
                }
            }
            Ok(_) => {}
            Err(e) => eprintln!("Watch error: {e}"),
        }
    }

    Ok(())
}
