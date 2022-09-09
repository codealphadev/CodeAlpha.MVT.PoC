import json
import os
import toml

package_file_path = "./package.json"
with open(package_file_path, "r") as jsonFile:
    data = json.load(jsonFile)
data["version"] = os.environ['VERSION']
with open(package_file_path, "w") as jsonFile:
    json.dump(data, jsonFile, indent=2)
print('Updated package.json version to ' + os.environ['VERSION'])

tauri_conf_file_path = "./src-tauri/tauri.conf.json"
with open(tauri_conf_file_path, "r") as jsonFile:
    data = json.load(jsonFile)
data["package"]["version"] = os.environ['VERSION']
with open(tauri_conf_file_path, "w") as jsonFile:
    json.dump(data, jsonFile, indent=2)
print('Updated tauri.conf.json version to ' + os.environ['VERSION'])

cargo_toml_file_path = "./src-tauri/Cargo.toml"
data = toml.load(cargo_toml_file_path)
data['package']['version'] = os.environ['VERSION']
f = open(cargo_toml_file_path, 'w')
toml.dump(data, f)
f.close()
print('Updated Cargo.toml version to ' + os.environ['VERSION'])
