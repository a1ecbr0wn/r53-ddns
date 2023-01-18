# Setup as a service on Linux using `systemd`

Create a file called `/etc/systemd/system/r53-ddns.service` as root.

``` service
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
