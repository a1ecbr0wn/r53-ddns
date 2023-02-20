---
layout: default
nav_order: 4
has_children: true
---

# Usage

r53-ddns can be used adhoc if you wish but you probably want to set this up to run continuously in case your external ip address changes.  There are two ways that you can do this, using a job scheduler such as cron or as a service.

- [Setup cron](setup-cron.md)
- [Setup Systemd Service on Linux](setup-systemd.md)
- [Setup Launchd Service on MacOS](setup-launchd.md)
