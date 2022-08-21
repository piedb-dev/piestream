#!/bin/bash

start_dns() {
       	pdsh -R ssh -l abc -w host[01-04] "echo 'nameserver 8.8.8.8' | sudo tee /etc/resolv.conf"
}

stop_dns() {
       	pdsh -R ssh -l abc -w host[01-04] "grep -v 8.8.8.8 /etc/resolv.conf > /tmp/resolv.conf; sudo cp /tmp/resolv.conf /etc/resolv.conf"
}

case "$1" in
	on) start_dns ;;
	off) stop_dns ;;
	*) echo "Usage: $0 [on | off]" ;;
esac
