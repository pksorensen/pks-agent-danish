#!/usr/bin/env python3
"""Install the pinned, checksum-verified Vale binary without changing PATH."""
import hashlib
import io
import platform
import tarfile
import urllib.request

from common import DATA, VERSION

ASSETS = {
    ("Linux", "x86_64"): ("Linux_64-bit", "4bf5c5176b682b1da0830f471584d3494f1c2e27ee3029e1ad6f6661d4962182"),
    ("Linux", "aarch64"): ("Linux_arm64", "2a69755b68118c612c455fc0c5ea1b9438519aa1c852e3fb303b3ad97141bbb0"),
    ("Darwin", "x86_64"): ("macOS_64-bit", "e27097e84a812d45546de8d6a4ffa75538cab39c96f79b153b601240f8cefaf9"),
    ("Darwin", "arm64"): ("macOS_arm64", "fbcaa68d9b9e229b9c8dc918f209e93572ce943039700b7e04a423ff12f684cd"),
}


def main():
    key = (platform.system(), platform.machine())
    if key not in ASSETS:
        raise SystemExit("Brug Linux/macOS eller WSL; se README om PKS_DANISH_VALE.")
    asset, expected = ASSETS[key]
    url = f"https://github.com/vale-cli/vale/releases/download/v{VERSION}/vale_{VERSION}_{asset}.tar.gz"
    directory = DATA / "tools" / VERSION
    directory.mkdir(parents=True, exist_ok=True)
    # Keep the bounded archive in memory; never extract arbitrary archive paths.
    with urllib.request.urlopen(url, timeout=60) as response:
        archive = response.read(32 * 1024 * 1024 + 1)
    if hashlib.sha256(archive).hexdigest() != expected:
        raise SystemExit("Forkert SHA-256; installation afbrudt.")
    with tarfile.open(fileobj=io.BytesIO(archive), mode="r:gz") as bundle:
        member = bundle.getmember("vale")
        if not member.isfile() or member.size > 64 * 1024 * 1024:
            raise SystemExit("Uventet binærfil i arkivet.")
        temporary = directory / "vale.download"
        temporary.write_bytes(bundle.extractfile(member).read())
    temporary.chmod(0o755)
    target = directory / "vale"
    temporary.replace(target)
    print(f"Vale {VERSION}: {target}")


if __name__ == "__main__":
    main()
