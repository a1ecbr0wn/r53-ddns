# Alert Script

A custom script can be specified on the command line which is then called whenever the ip address record in the dns record is changed or when an error occurs.  This feature adds customised actions to be run following the events e.g. alerting, automatic routing changes, etc.

The script is called with one parameter that is a json document in one of the following formats:

## IP Address Change

## Error
