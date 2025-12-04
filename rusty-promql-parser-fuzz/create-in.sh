#!/bin/bash

mkdir -p in

num=1
while IFS= read -r line || [[ -n "$line" ]]; do
    echo -n "$line" > "in/prom${num}"
    ((num++))
done < src/in.txt
