---
layout: default
nav_order: 1
parent: Usage
---

# Setup as a service on Linux using `systemd`

Create a file called `/etc/systemd/system/r53-ddns.service` as root.

``` sh
[Unit]
Description=R53 DDNS Service
After=network.target
StartLimitIntervalSec=0

[Service]
Type=simple
Restart=always
RestartSec=1
User=root
ExecStart=/snap/bin/r53-ddns -s=net -d=example.com -c=300

[Install]
WantedBy=multi-user.target
```

Set up the AWS credentials file in default location for the root user.

Start the service with the following:

``` sh
sudo systemctl start r53-ddns
```

Make sure it restarts on reboot:

``` sh
sudo systemctl daemon-reload
sudo systemctl enable r53-ddns
```

Check it is running OK:

``` sh
systemctl status r53-ddns
```

Another issue you will find it that the binary snap will not update unless you stop the service.  You might want to consider putting something like this in the crontab of your root user which will keep all of your :

``` sh
0 0 0 * * service r53-ddns stop && snap refresh r53-ddns && service r53-ddns start
```
