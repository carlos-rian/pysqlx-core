import httpx
import toml

with open("Cargo.toml", mode="r") as file:
    text: str = file.read()


def get_version():
    for _ in range(3):
        resp = httpx.get("https://pypi.org/pypi/pysqlx-core/json")
        if resp.status_code == 200:
            break
    data: dict = resp.json()

    releases = data["releases"]
    versions = sorted(
        releases.keys(),
        key=lambda x: int(x.replace(".", "").replace("b", "").replace("a", "")),
        reverse=True,
    )
    versions = [v for v in versions if "b" in v]

    current_version = data["info"]["version"]

    if versions:
        if int(current_version.replace(".", "")) > int(
            versions[0].split("b")[0].replace(".", "")
        ):
            return current_version
        return versions[0]

    return current_version


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

print("Package new version:", new_version)

new_text = text.replace(f'version = "{file_version}"', f'version = "{new_version}"')

if new_version not in new_text:
    raise Exception("Could not update version, check the Cargo.toml file.")

with open("Cargo.toml", mode="w") as file:
    file.write(new_text)
