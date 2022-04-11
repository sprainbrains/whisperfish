#!/bin/bash
# Script to determine if the parameter version string
# is the same or newer than the current build target.
grep VERSION_ID /etc/sailfish-release | cut -d "=" -f2 > _target
echo "$1" >> _target
FIRST=$(sort -V < _target | head -1) 
rm _target
if [[ "$FIRST" == "$1" ]]
then
  echo 1
else
  echo 0
fi
