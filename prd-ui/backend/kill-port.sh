#!/bin/bash
# Kill process on port 3001

PORT=${1:-3001}
PID=$(lsof -ti:$PORT)

if [ -z "$PID" ]; then
  echo "No process found on port $PORT"
  exit 0
fi

echo "Killing process $PID on port $PORT"
kill -9 $PID
echo "Done"
