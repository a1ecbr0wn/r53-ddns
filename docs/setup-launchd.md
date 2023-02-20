---
layout: default
nav_order: 3
parent: Usage
---

# Setup as a service on MacOS using `launchd`

Create a file called `~/Library/LaunchAgents/r53-ddns.plist` as the current user:

``` xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>r53-ddns</string>
    <key>ProgramArguments</key>
    <array>
        <string>/opt/homebrew/bin/r53-ddns</string>
        <string>-s=net</string>
        <string>-d=example.com</string>
        <string>-c=300</string>
    </array>
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>
```

Set up the AWS credentials file in default location for the current user.

To load and start your service run the folowing:

``` sh
launchctl load ~/Library/LaunchAgents/r53-ddns.plist
```

The service should startup each time the user logs in
