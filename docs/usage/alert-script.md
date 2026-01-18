---
layout: docs
title: "Alert Script| r53-ddns"
---

## Alert Script

A custom script can be specified on the command line which is then called
whenever the ip address record in the dns record is changed or when an error
occurs. This feature adds customised actions to be run following the events
e.g. alerting, automatic routing changes, etc.

The script is called with one parameter that is a json document in one of the
following formats:

### IP Address New

`{ "type": "ip-new", "dns": "host.example.com.", "new": "10.205.309.35" }`

### IP Address Change

`{ "type": "ip-change", "dns": "host.example.com.", "old": "10.205.309.34",
"new": "10.205.309.35" }`

### Error

`"{ "type": "error", "msg": "error message..." }"`
