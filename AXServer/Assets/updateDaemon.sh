#!/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

launchctl unload ~/Library/LaunchAgents/com.codeAlpha.AXServerXPC.plist

sleep .5

cp $SCRIPT_DIR/com.codeAlpha.AXServerXPC.plist ~/Library/LaunchAgents/

sleep .5

launchctl load ~/Library/LaunchAgents/com.codeAlpha.AXServerXPC.plist