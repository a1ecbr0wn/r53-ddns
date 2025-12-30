---
layout: docs
title: "Usage | r53-ddnsß"
---

# Usage

`r53-ddns` can be used adhoc if you wish but you probably want to set this up to run continuously in case your external ip address changes.  There are two ways that you can do this, using a job scheduler such as cron or as a service.

## Setup

- [Setup with cron](setup-cron)
- [Setup as a service on Linux using `systemd`](setup-systemd)
- [Setup as a service on MacOS using `launchd`](setup-launchd)
- [Alert Script](alert-script)
