---
layout: docs
title: "Setup with cron | r53-ddns"
---

# Setup with cron

The following example is an entry into a cron file that will set up the subdomain `net.example.com`, performing the external ip and dns check every 5 minutes, assuming that the application has been installed via snap:

``` text
*/5 * * * * /snap/bin/r53-ddns -s=net -d=example.com
```

At the top of the cron file, you may also want to declare the AWS environment variables that provide the credentials:

``` text
AWS_ACCESS_KEY = ...
AWS_SECRET_ACCESS_KEY = ...
```

Some of the default services used to return the external ip address of your network will stop giving you a response if called too frequently, it is recomended that you don't call them more often than once every 5 minutes without increasing the number of configured services (via the `-i` parameter), hence the `*/5` in the cron example above.

As an added bonus, if you have email set up on your server you can set up a `MAILTO` environment variable in your crontab and it will email you every time your ip address changes.
