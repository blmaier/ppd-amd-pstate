#!/bin/bash

systemctl disable --now ppd-amd-pstate.service
rm -f /etc/systemd/system/ppd-amd-pstate.service
rm -f /usr/sbin/ppd-amd-pstate
