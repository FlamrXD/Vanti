from libs.filesearch import watch_downloads
from libs.hashd import hashcheck

for file_path in watch_downloads():
    
    print("Received file path:", repr(file_path))
    hashcheck(file_path)