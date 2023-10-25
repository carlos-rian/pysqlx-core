# git: https://github.com/prisma/prisma-engines.git

# download quaint from https://github.com/prisma/prisma-engines.git
# save zip file to /tmp
# unzip to /tmp
# copy quaint from tmp to ./quaint
# delete zip file and tmp folder


import shutil
import subprocess
from pathlib import Path

import fsspec

# create temp folder
destination = Path("/tmp/prisma")
shutil.rmtree(destination, ignore_errors=True)
destination.mkdir(exist_ok=True, parents=True)

# download quaint
url = "https://github.com/prisma/prisma-engines.git"
subprocess.run(["git", "clone", "--depth", "1", url, "/tmp/prisma"], check=True)


# copy quaint from tmp to ./quaint
shutil.copytree("/tmp/prisma/quaint/", "./quaint")

# delete zip file and tmp folder
shutil.rmtree("/tmp/prisma")
