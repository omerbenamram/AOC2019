import os
import subprocess
from pathlib import Path

if __name__ == '__main__':
    for (r, d, files) in os.walk(Path('.')):
        for f in files:
            if f.endswith('.ps1'):
                subprocess.run(["powershell", "-NoProfile", f".\{f}"], cwd=r)
