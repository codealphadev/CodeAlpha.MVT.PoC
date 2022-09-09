import os
import datetime
import requests
import json

with open('./src-tauri/target/universal-apple-darwin/release/bundle/macos/CodeAlpha.app.tar.gz.sig', 'r') as file:
    signature = file.read()
# TODO: CHANGE WHEN SIGNATURE SHOULD BE READ
#signature = "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUlVUdm5GMlA5eGZCVHJIYkowQ25PMDNKSGtHRWIyYXp5NVpJRUxHSFhyM2R2VWF3SDJNSHg5dnQxZXc2cUVQRGVYZTBrMDF0d1ZybFVndkE0clJxWm1Fc3Y5R3h2cUVHQnd3PQp0cnVzdGVkIGNvbW1lbnQ6IHRpbWVzdGFtcDoxNjYyNDU2MzQ0CWZpbGU6L1VzZXJzL2FkYW0vY29kZWFscGhhL2NvZGUvQ29kZUFscGhhLk1WVC5Qb0MvYWN0aW9ucy1ydW5uZXIvX3dvcmsvQ29kZUFscGhhLk1WVC5Qb0MvQ29kZUFscGhhLk1WVC5Qb0Mvc3JjLXRhdXJpL3RhcmdldC91bml2ZXJzYWwtYXBwbGUtZGFyd2luL3JlbGVhc2UvYnVuZGxlL21hY29zL0NvZGVBbHBoYS5hcHAudGFyLmd6CkcyL29NYXZvYk9wZW1weVMvVTFqd0NERHArdEthUlJMUWlMQWx6QmQrOU5meGpuVHEzdnBBbVY2SGJiU1VjajRmbTIzUUNTK0ZoOTdjVUM5WUthZUFnPT0K"

url = "https://storage.googleapis.com/codealpha-releases/{}/macos/CodeAlpha.app.tar.gz".format(
    os.environ['VERSION'])
release = {
    "version": os.environ['VERSION'],
    "pub_date": datetime.datetime.now().isoformat(),
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
print(release)

f = open("manifest.json", "a")
f.write(json.dumps(release))

print("Updated release manifest")
