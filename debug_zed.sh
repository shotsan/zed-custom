#!/bin/bash
# Enable debug logs
export RUST_LOG=agent=debug,info

# Clear the log file so we see only new logs
echo "" > ~/Library/Logs/Zed/Zed.log

echo "Starting Zed... Logs will be streamed from ~/Library/Logs/Zed/Zed.log"
echo "Waiting for: MemoryDatabase, memories, Remember, NativeAgent, panic, ERROR"
echo "---------------------------------------------------"

# Start tailing the log file in the background
tail -f ~/Library/Logs/Zed/Zed.log | grep --line-buffered -E "MemoryDatabase|memories|Remember|NativeAgent|panic|ERROR" &
TAIL_PID=$!

# Run Zed
./target/debug/zed

# Kill tail when Zed exits
kill $TAIL_PID
