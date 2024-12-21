#!/bin/bash
set -e

echo "Starting Rafka retention policy demo..."

# Start broker with custom retention policy (10 seconds)
echo "Starting broker with 10-second retention..."
cargo run -q broker --retention-secs 10 &
BROKER_PID=$!
sleep 2

# Start consumer
echo "Starting consumer..."
cargo run -q consumer &
CONSUMER_PID=$!
sleep 2

# Send first batch of messages
echo "Sending first batch of messages..."
for i in {1..5}; do
    cargo run -q producer \
        --message "Message$i" \
        --key "key-$i"
    sleep 1
done

# Wait for retention period
echo "Waiting for retention period (12 seconds)..."
sleep 12

# Check storage metrics
echo "Checking storage metrics..."
cargo run -q metrics 

# Send second batch of messages
echo "Sending second batch of messages..."
for i in {6..10}; do
    cargo run -q producer \
        --message "Message$i" \
        --key "key-$i"
    sleep 1
done

# Clean up
echo "Cleaning up..."
kill $BROKER_PID $CONSUMER_PID

echo "Retention policy demo completed" 
