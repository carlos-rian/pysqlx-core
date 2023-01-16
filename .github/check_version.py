import httpx
import toml
import os

with open("Cargo.toml", mode="r") as file:
    text: str = file.read()


def get_version():
    uri = "https://pypi.org/pypi/pysqlx-core/json"
    for _ in range(3):
        resp = httpx.get(uri)
        if resp.is_success:
            break
    json: dict = resp.json()
    return json["info"]["version"]


version: str = get_version()
v = toml.loads(text)["package"]["version"]
file_version = v.replace("-alpha", "a").replace("-beta", "b")

print("Package version:", version)

MAJOR, MINOR, PATCH = version.split(".")

PATCH = int(PATCH) + 1

new_version: str = ".".join([MAJOR, MINOR, str(PATCH)])

print("Package new version:", new_version)

new_text = text.replace(f'version = "{file_version}"', f'version = "{new_version}"')

with open("Cargo.toml", mode="w") as file:
    file.write(new_text)


env_file = os.getenv('GITHUB_ENV')

with open(env_file, mode="a") as file:
    file.write(f"\nPY_SQLX_VERSION=v{new_version}")
