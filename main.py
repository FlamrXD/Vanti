from pathlib import Path
from libs.filesearch import watch_downloads
from libs.hashd import hashcheck
from plyer import notification
import time
import os
PROJECT_DIR = Path(__file__).parent.resolve()

print(f"Monitoring Downloads folder: ")
notification.notify(
    title="Vanti",
    message=f"Vanti is enabled, keeping you secured",
    app_name="Vanti",
    timeout=5,
)
for file_path in watch_downloads():
    if file_path.name == "hashes.txt":
        continue
    time.sleep(2)
    print("Received file path:", repr(file_path))
    is_virus = hashcheck(file_path)
    print(f"{file_path} is a virus: {is_virus}")
    if is_virus:
        os.remove(file_path)
        notification.notify(
            title="Virus Contained",
            message=f"The file {file_path} has been detected as a virus and has been contained. Move it out of the containment folder to keep it or else it will be deleted in 24 Hours",
            app_name="Vanti",
            timeout=5,
        )