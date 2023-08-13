#!/bin/bash

./uninstall.sh

cp ./ppd-amd-pstate.service /etc/systemd/system/
install ./ppd-amd-pstate.sh /usr/sbin/ppd-amd-pstate

systemctl enable --now ppd-amd-pstate.service
