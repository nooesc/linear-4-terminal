#!/bin/bash

# Test script to reproduce the crash
echo "This script will help reproduce the crash when pressing 'p' to edit project"
echo ""
echo "Instructions:"
echo "1. The Linear CLI will start in interactive mode"
echo "2. Navigate to any issue using j/k or arrow keys"
echo "3. Press 'p' to edit the project"
echo "4. Press 'l' to edit labels"
echo "5. If it crashes, check the log file afterwards"
echo ""
echo "Log file location: ~/.cache/linear-cli/logs/"
echo ""
echo "Press Enter to start..."
read

# Run the linear CLI
linear interactive

# After exit, check for crash log
echo ""
echo "Linear CLI exited."
if [ -f "/tmp/linear-cli-last-log.txt" ]; then
    echo "Crash detected! Log file: $(cat /tmp/linear-cli-last-log.txt)"
    rm /tmp/linear-cli-last-log.txt
else
    echo "Latest log: ~/.cache/linear-cli/logs/$(ls -t ~/.cache/linear-cli/logs/ | head -1)"
fi