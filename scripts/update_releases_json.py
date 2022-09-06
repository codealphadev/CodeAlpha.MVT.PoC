import os
import datetime
import requests
import json

with open('./src-tauri/target/universal-apple-darwin/release/bundle/macos/CodeAlpha.app.tar.gz.sig', 'r') as file:
    signature = file.read()

response = requests.get("https://storage.googleapis.com/codealpha-releases/releases.json")
with open("./releases.json", "w") as f: # opening a file handler to create new file 
    f.write(resp.content) 

release = {
    "version": "${{ steps.versioning.outputs.version }}",
    "pub_date": datetime.datetime.now().isoformat(),
    "platforms": {
        "darwin-aarch64": {
            "signature": signature,
            "url": "${{ steps.upload-file.outputs.uploaded }}"
        },
        "darwin-x86_64": {
            "signature": signature,
            "url": "${{ steps.upload-file.outputs.uploaded }}"
        }
    }
}

f = open("releases.json", "a")
f.write(json.dumps(release))