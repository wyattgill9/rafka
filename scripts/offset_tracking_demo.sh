#!/bin/bash
set -e

echo "Starting Rafka offset tracking demo..."

# Start broker
echo "Starting broker..."
cargo run broker &
BROKER_PID=$!
sleep 2

# Start first consumer
echo "Starting first consumer..."
cargo run consumer &
CONSUMER1_PID=$!
sleep 2

# Send initial batch of messages
echo "Sending first batch of messages..."
for i in {1..3}; do
    cargo run producer \
        --message "Batch1-Message$i" \
        --key "key-$i"
    sleep 1
done

# Let consumer process messages and update offsets
sleep 2

# Restart consumer to demonstrate offset tracking
echo "Restarting consumer..."
kill $CONSUMER1_PID
sleep 2

echo "Starting second consumer with same ID..."
cargo run consumer &
CONSUMER2_PID=$!
sleep 2

# Send second batch of messages
echo "Sending second batch of messages..."
for i in {4..6}; do
    cargo run producer \
        --message "Batch2-Message$i" \
        --key "key-$i"
    sleep 1
done

# Clean up
echo "Cleaning up..."
kill $BROKER_PID $CONSUMER2_PID

echo "Offset tracking demo completed" 
