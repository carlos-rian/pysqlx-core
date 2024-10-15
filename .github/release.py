import httpx
import toml

with open("Cargo.toml", mode="r") as file:
    text: str = file.read()


def get_version():
    uri = "https://pypi.org/pypi/pysqlx-core/json"
    for _ in range(3):
        resp = httpx.get(uri)
        if resp.is_success:
            break
    json: dict = resp.json()

    releases = json["releases"]
    versions = sorted(
        releases.keys(),
        key=lambda x: int(x.replace(".", "").replace("b", "")),
        reverse=True,
    )
    versions = [v for v in versions if v.startswith("0.2.0") and "b" in v]
    return versions.pop() if versions else "0.2.0b-1"


version: str = get_version()
file_version = toml.loads(text)["package"]["version"]

print("Package version:", version)

# MAJOR, MINOR, PATCH = version.replace("b", "").split(".")
MAJOR, MINOR, PATCH = version.split(".")
F_MAJOR, F_MINOR, F_PATCH = file_version.split(".")

IS_FILE = False
if MAJOR < F_MAJOR or MINOR < F_MINOR:
    MAJOR, MINOR, PATCH = F_MAJOR, F_MINOR, F_PATCH
    IS_FILE = True


if "b" in PATCH:
    PATCH, BETA = PATCH.split("b")
    BETA = int(BETA) + 1
elif IS_FILE:
    BETA = 0
else:
    PATCH = int(PATCH) + 1
    BETA = 0

PATCH = "".join([str(PATCH), "-beta", str(BETA)])

new_version: str = ".".join([MAJOR, MINOR, str(PATCH)])

print("Package new version: ", new_version)

new_text = text.replace(f'version = "{file_version}"', f'version = "{new_version}"')

if new_version not in new_text:
    raise Exception("Could not update version, check the Cargo.toml file.")

with open("Cargo.toml", mode="w") as file:
    file.write(new_text)
