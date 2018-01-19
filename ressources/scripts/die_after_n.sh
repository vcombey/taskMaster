#!/usr/bin/env bash

if [ -n $1 ]; then
	a=$1
	while [ "$a" -ne "0" ]; do echo $a... ; sleep 1 ; a=$(($a - 1)) ; done
	echo "Dying now"
else
	echo Specify number of sec to sleep 1>&2
fi
