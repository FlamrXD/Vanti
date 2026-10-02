import hashlib
import os

BASE_DIR = os.path.dirname(os.path.abspath(__file__))
HASH_LIST_PATH = os.path.join(
    BASE_DIR, "hashes.txt"
)


def hashcheck(fileToCheck):
  clean_path = str(fileToCheck).strip().strip('"').strip("'")

  if not os.path.isfile(clean_path):
    print(f"Error: Path does not exist or is a directory: {clean_path}")
    return False

  with open(clean_path, "rb") as f:
    digest = hashlib.file_digest(f, "sha256")
    file_hash = digest.hexdigest()

  print(f"File Hash: {file_hash}")

  if not os.path.exists(HASH_LIST_PATH):
    print(f"Error: Hash reference list missing at {HASH_LIST_PATH}")
    return False

  with open(HASH_LIST_PATH, "r", encoding="utf-8") as file:
    content = file.read()
    return file_hash.lower() in content.lower()