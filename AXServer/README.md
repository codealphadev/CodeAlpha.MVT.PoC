
## Installation instructions

1. Clone Git repository, navigate to repository's root folder
2. Build the project: `swift build`
3. Execute `sh Assets/updateDaemon.sh` to load `*plist` file to `launchd`. This registers this service to be launched at user login. 
4. Check if the daemon runs without errors (for now...): `launchctl list | grep codeAlpha`. If there is an error, go investigate. 🤓

## Accepted Constraints

* [02.03.2022]: The folder where the executable is expected by the `launchd` agent is still specific to the user's file system. 
* [02.03.2022]: On first startup and after granting accessibility api permissions, the service might not serve the right content to consumers
* [02.03.2022]: Websocket URL (ws://127.0.0.1:8080/channel) is still hard coded. We might change it to be set via CLI arguments
* [02.03.2022]: No TLS via websocket
* [02.03.2022]: No authentication