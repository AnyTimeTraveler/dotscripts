#!/usr/bin/env zsh

value="$1" 
echo "Getting display count"
display_count="$(ddcutil --brief detect | grep -P '^Display \d' | wc -l)"
echo "Found ${display_count} displays"
for i in {1..$display_count}; do
	echo "Setting display ${i} to ${value}%"
	ddcutil -d $i setvcp 10 "$value"
done
