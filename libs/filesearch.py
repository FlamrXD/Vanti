from queue import Queue, Empty
from pathlib import Path
from typing import Generator, Optional
from watchdog.observers import Observer
from watchdog.events import FileSystemEventHandler


class _NewFileHandler(FileSystemEventHandler):
    def __init__(self, queue: Queue):
        super().__init__()
        self.queue = queue

    def on_created(self, event):
        if not event.is_directory:
            self.queue.put(Path(event.src_path))


def watch_downloads(
    path: Optional[Path] = None,
    poll_interval: float = 0.5,
) -> Generator[Path, None, None]:
    """
    Yields the Path object of each newly detected file as soon as it is created.

    :param path: Path object or string of the directory to watch. Defaults to ~/Downloads.
    :param poll_interval: Timeout in seconds for checking events (allows clean exit on Ctrl+C).
    """
    target_path = Path(path) if path else (Path.home() / "Downloads")
    file_queue: Queue = Queue()

    handler = _NewFileHandler(file_queue)
    observer = Observer()
    observer.schedule(handler, path=str(target_path), recursive=False)
    observer.start()

    try:
        while True:
            try:
                file_path = file_queue.get(timeout=poll_interval)
                yield file_path
            except Empty:
                continue
    except KeyboardInterrupt:
        pass
    finally:
        observer.stop()
        observer.join()