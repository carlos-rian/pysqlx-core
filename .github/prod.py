import http.client as http
import json
import os

import toml

with open("Cargo.toml", mode="r") as file:
    text: str = file.read()


# convert get_version to use native python libs
def get_version():
    host = "pypi.org"
    for _ in range(3):
        conn = http.HTTPSConnection(host, timeout=30)
        conn.request("GET", "/pypi/pysqlx-core/json")
        resp = conn.getresponse()
        if resp.status == 200:
            break
    data: dict = json.load(resp)
    return data["info"]["version"]


version: str = get_version()
file_version = toml.loads(text)["package"]["version"]

print("Package version:", version)

MAJOR, MINOR, PATCH = version.split(".")

PATCH = int(PATCH) + 1

new_version: str = ".".join([MAJOR, MINOR, str(PATCH)])

print("Package new version:", new_version)

new_text = text.replace(f'version = "{file_version}"', f'version = "{new_version}"')

if new_version not in new_text:
    raise Exception("Could not update version, check the Cargo.toml file.")

with open("Cargo.toml", mode="w") as file:
    file.write(new_text)

env_file = os.getenv('GITHUB_ENV')

with open(env_file, mode="a") as file:
    file.write(f"\nPY_SQLX_VERSION=v{new_version}")
