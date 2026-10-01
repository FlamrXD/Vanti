import hashlib

def hashcheck(fileToCheck):
    with open(fileToCheck, "rb") as f:
        digest = hashlib.file_digest(f, "sha256")
        file_hash = digest.hexdigest()
    print(file_hash)

hashcheck("ha.txt")


