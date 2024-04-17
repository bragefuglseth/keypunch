#!/bin/bash

for file in "$@"
do
    jq -r '.words[]' $file > $file.tmp && mv $file.tmp $file
done