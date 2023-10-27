import json
import urllib

import toml

with open("Cargo.toml", mode="r") as file:
    text: str = file.read()


def get_version():
    uri = "https://pypi.org/pypi/pysqlx-core/json"
    for _ in range(3):
        resp = urllib.request.urlopen(uri)
        if resp.status == 200:
            break
    data: dict = json.loads(resp.read())

    releases = data["releases"]
    versions = sorted(
        releases.keys(), key=lambda x: int(x.replace(".", "").replace("b", "").replace("a", "")), reverse=True
    )
    versions = [v for v in versions if "b" in v]

    current_version = data["info"]["version"]

    if versions:
        if int(current_version.replace(".", "")) > int(versions[0].split("b")[0].replace(".", "")):
            return current_version
        return versions[0]

    return current_version


version: str = get_version()
file_version = toml.loads(text)["package"]["version"]

print("Package version:", version)

# MAJOR, MINOR, PATCH = version.replace("b", "").split(".")
MAJOR, MINOR, PATCH = version.split(".")

if "b" in PATCH:
    PATCH, BETA = PATCH.split("b")
    BETA = int(BETA) + 1
else:
    PATCH = int(PATCH) + 1
    BETA = 0

PATCH = "".join([str(PATCH), "-beta", str(BETA)])

new_version: str = ".".join([MAJOR, MINOR, str(PATCH)])

print("Package new version:", new_version)

new_text = text.replace(f'version = "{file_version}"', f'version = "{new_version}"')

if new_version not in new_text:
    raise Exception("Could not update version, check the Cargo.toml file.")

with open("Cargo.toml", mode="w") as file:
    file.write(new_text)
