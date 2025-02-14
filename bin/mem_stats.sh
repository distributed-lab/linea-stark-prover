#!/bin/bash

# File to log memory statistics
LOG_FILE="memory_stats.log"

# Clear the log file before starting
> "$LOG_FILE"

# Run your program in the background
cargo run --release --features parallel &

# Get the PID of the program
PID=$!

echo $PID

# Initialize maximum memory usage
MAX_MEMORY_GB=0

# Log memory usage until the program exits
while kill -0 "$PID" 2>/dev/null; do
    # Get memory usage for the process in KB
    MEMORY_USAGE_KB=$(ps -o rss= -p "$PID")
    # Convert KB to GB (divide by 1,048,576)
    MEMORY_USAGE_GB=$(echo "scale=6; $MEMORY_USAGE_KB / 1048576" | bc)
    # Update maximum memory usage if current usage is higher
    MAX_MEMORY_GB=$(echo "scale=6; if ($MEMORY_USAGE_GB > $MAX_MEMORY_GB) $MEMORY_USAGE_GB else $MAX_MEMORY_GB" | bc)
    # Get timestamp
    TIMESTAMP=$(date +"%Y-%m-%d %H:%M:%S")
    # Write to log file
    echo "$TIMESTAMP Memory: ${MEMORY_USAGE_GB} GB" >> "$LOG_FILE"
    # Sleep for 1 second before checking again
    sleep 1
done

# Log the maximum memory usage
echo "Maximum Memory Used: ${MAX_MEMORY_GB} GB" >> "$LOG_FILE"

echo "Program has exited. Memory monitoring stopped."
