# Linear CLI Debugging Guide

## How to Test and Debug the Crash

### 1. Running the Interactive Mode

```bash
# Run the test script
./test_crash.sh

# Or run directly
linear interactive
```

### 2. Reproducing the Crash

1. Once in interactive mode, navigate to any issue using `j`/`k` keys
2. Press `p` to edit the project field
3. If it crashes, note the exact behavior

### 3. Checking the Logs

The application now logs detailed information to help debug crashes:

```bash
# Find the latest log file
ls -t ~/.cache/linear-cli/logs/ | head -1

# View the log
cat ~/.cache/linear-cli/logs/linear-*.log
```

### 4. What the Logs Show

- `[INFO]` - High-level operations
- `[DEBUG]` - Detailed state information
- `[ERROR]` - Errors and panics

Key log entries to look for:
- "Handling 'p' key for project edit" - When 'p' is pressed
- "Selected issue:" - Shows which issue was selected
- "Available projects:" - Shows how many projects are available
- "Rendering project selection" - When UI tries to render the dialog
- "PANIC:" - If a panic occurs

### 5. Common Issues and Solutions

#### Issue: Crash when pressing 'p'
**Check:**
- Is `selected_issue_id` being set correctly?
- Is `option_index` within bounds?
- Are there any projects available?

#### Issue: Crash when pressing 'l' for labels
**Check:**
- Similar issues as with projects
- Label IDs being correctly populated

### 6. Testing Label Functionality

Since you mentioned labels are also broken, test:
1. Press `l` on any issue
2. Check if it crashes or shows the label selection dialog

### 7. If You Need More Debugging

Add more debug output by editing the code:
- Add `log_debug()` calls in suspicious areas
- Check array bounds before accessing
- Verify state transitions

### 8. Send Me the Logs

If it crashes, please share:
1. The exact steps you took
2. The relevant portion of the log file
3. Any error messages shown

The logs will help identify exactly where and why it's crashing.