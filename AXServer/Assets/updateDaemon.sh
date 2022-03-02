#!/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

launchctl unload ~/Library/LaunchAgents/com.codeAlpha.AXServer.plist

sleep .5

cp $SCRIPT_DIR/com.codeAlpha.AXServer.plist ~/Library/LaunchAgents/

sleep .5

launchctl load ~/Library/LaunchAgents/com.codeAlpha.AXServer.plist