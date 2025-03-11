#!/bin/bash

# Check if a file name was provided
if [ -z "$1" ]; then
  echo "Usage: $0 <file_name>"
  exit 1
fi

# Assign the file name to a variable
file_name="$1"

# Initialize total time in seconds
total_time=0

# Read each line from the specified file
while IFS= read -r line; do
  # Extract the time value using regex
  if [[ $line =~ prove\ \[\ ([0-9.]+)([a-z]+)\ \| ]]; then
    time_value=${BASH_REMATCH[1]}
    time_unit=${BASH_REMATCH[2]}

    # Convert the time to seconds based on the unit
    case $time_unit in
      s)
        total_time=$(echo "$total_time + $time_value" | bc)
        ;;
      ms)
        total_time=$(echo "$total_time + $time_value / 1000" | bc)
        ;;
      m)
        total_time=$(echo "$total_time + $time_value * 60" | bc)
        ;;
      h)
        total_time=$(echo "$total_time + $time_value * 3600" | bc)
        ;;
    esac
  fi
done < "$file_name"

# Print the total time in seconds
echo "Total time: $total_time seconds"
