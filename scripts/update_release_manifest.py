import os
import datetime
import json

with open('./src-tauri/target/universal-apple-darwin/release/bundle/macos/Pretzl.app.tar.gz.sig', 'r') as file:
    signature = file.read()

url = "https://storage.googleapis.com/pretzl-releases/{}/macos/Pretzl.app.tar.gz".format(
    os.environ['VERSION'])
release = {
    "version": os.environ['VERSION'],
    "pub_date": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "platforms": {
        "darwin-aarch64": {
            "signature": signature,
            "url":  url
        },
        "darwin-x86_64": {
            "signature": signature,
            "url": url
        }
    }
}
release_string = json.dumps(release, indent=2)
print(release_string)

f = open("manifest.json", "a")
f.write(release_string)

print("Updated release manifest")
