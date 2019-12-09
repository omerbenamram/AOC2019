import glob
import os
import subprocess

if __name__ == '__main__':
    for p in glob.glob(r"**\*.ps1"):
        root, base = os.path.dirname(p), os.path.basename(p)
        subprocess.run(["powershell", "-NoProfile", f".\{base}"], cwd=root)
