import json
import os
import toml

print("Updating repo version to ", os.environ["VERSION"])

package_file_path = "./package.json"
with open(package_file_path, "r") as jsonFile:
    data = json.load(jsonFile)
data["version"] = os.environ['VERSION']
with open(package_file_path, "w") as jsonFile:
    jsonFile.write(json.dumps(data, indent=2) + '\n')
print('Updated package.json version to ' + os.environ['VERSION'])

tauri_conf_file_path = "./src-tauri/tauri.conf.json"
with open(tauri_conf_file_path, "r") as jsonFile:
    data = json.load(jsonFile)
data["package"]["version"] = os.environ['VERSION']
with open(tauri_conf_file_path, "w") as jsonFile:
    jsonFile.write(json.dumps(data, indent=2) + '\n')

cargo_toml_file_path = "./src-tauri/Cargo.toml"
with open(cargo_toml_file_path, "r") as tomlFile:
    data = toml.load(tomlFile)
data['package']['version'] = os.environ['VERSION']
with open(cargo_toml_file_path, "w") as tomlFile:
    tomlFile.write(toml.dumps(data) + '\n')
print('Updated Cargo.toml version to ' + os.environ['VERSION'])
