# ICS Calendar Setup Guide

This guide will help you set up ICS file integration for your Calendar Monitor.

## What is ICS?

ICS (iCalendar) is a standard calendar data exchange format used by most calendar applications. You can export ICS files from:
- Google Calendar
- Outlook/Exchange
- Apple Calendar
- Most other calendar applications

## Quick Setup

1. **Export your calendar as an ICS file**
2. **Place the ICS file in your project directory**
3. **Set the environment variable**
4. **Run the application**

## Step 1: Export Calendar as ICS

### From Google Calendar:
1. Go to [Google Calendar](https://calendar.google.com)
2. On the left sidebar, find your calendar
3. Click the three dots next to your calendar name
4. Select "Settings and sharing"
5. Scroll down to "Integrate calendar"
6. Copy the "Public address in iCal format" URL, or
7. Click "Export" to download an ICS file

### From Outlook:
1. Open Outlook
2. Go to File > Open & Export > Import/Export
3. Choose "Export to a file"
4. Select "iCalendar Format (.ics)"
5. Choose your calendar and save the file

### From Apple Calendar:
1. Open Calendar app
2. Select the calendar you want to export
3. File > Export > Export...
4. Save as .ics file

## Step 2: Configure Environment Variables

Configure your calendar sources in `calendar-monitor.toml` or environment variables:

### Single Calendar

**Option 1: TOML Configuration**
```toml
[ics]
file_paths = ["./my_calendar.ics"]
```

**Option 2: Environment Variable**
```bash
export ICS_FILE_PATH=./my_calendar.ics
```

### Multiple Calendars
```bash
# Multiple ICS files (comma-separated)
ICS_FILE_PATHS=./work_calendar.ics,./personal_calendar.ics,./project_calendar.ics
```

### Mixed Sources
```bash
# You can combine local files and URLs
ICS_FILE_PATHS=./work.ics,https://calendar.google.com/calendar/ical/personal@gmail.com/public/basic.ics
```

You can use:
- **Local file paths**: `./calendar.ics`
- **URLs to live ICS feeds**: `https://calendar.google.com/calendar/ical/your_id/public/basic.ics`
- **Mix of both**: Local files and URLs in the same configuration

## Step 3: Setup Your Calendar Source

### Option A: Local ICS Files
If you downloaded ICS files from your calendar provider, place them in your project directory:

**Single Calendar:**
```
calendar-monitor/
├── calendar.ics          # Your downloaded ICS file
├── calendar-monitor.toml # Configuration file
├── Cargo.toml
└── src/
```

**Multiple Calendars:**
```
calendar-monitor/
├── work_calendar.ics     # Work meetings
├── personal_calendar.ics # Personal events
├── project_calendar.ics  # Project deadlines
├── calendar-monitor.toml # Configuration with multiple calendars
├── Cargo.toml
└── src/
```

### Option B: Live ICS URLs
If you're using live calendar URLs (like Google Calendar public feeds), **no file placement needed!** Just configure your calendar sources:

**Option 1: TOML Configuration**
```toml
[ics]
# Single live calendar
file_paths = ["https://calendar.google.com/calendar/ical/youremail@gmail.com/public/basic.ics"]

# Or multiple live calendars
file_paths = [
  "https://work-calendar-url/basic.ics",
  "https://personal-calendar-url/basic.ics"
]
```

**Option 2: Environment Variables**
```bash
# Single live calendar
export ICS_FILE_PATH=https://calendar.google.com/calendar/ical/youremail@gmail.com/public/basic.ics

# Multiple live calendars
export ICS_FILE_PATHS=https://work-calendar-url/basic.ics,https://personal-calendar-url/basic.ics
```

**Project structure for live URLs:**
```
calendar-monitor/
├── calendar-monitor.toml # URLs only, no ICS files needed
├── Cargo.toml
└── src/
```

## Keeping Your Calendar Data Fresh

### Live URLs (Recommended)
If you're using live ICS URLs (Option B above), your calendar data updates automatically every time the application fetches from the URL. **No additional setup needed!**

### Local Files - Update Options

**Manual Updates:**
Periodically re-export and replace your ICS files when you want fresh calendar data.

**Automated Sync (Advanced):**
Create a script to periodically download the latest ICS file:

```bash
#!/bin/bash
# sync_calendar.sh
curl -o calendar.ics "https://your-calendar-url/basic.ics"
```

Run this script periodically with cron to keep local files up to date.

## Multiple Calendar Benefits

✅ **Separate Work & Personal**: Keep work meetings and personal events in different files  
✅ **Project Organization**: Dedicated calendars for different projects or teams  
✅ **Mixed Sources**: Combine local files with live calendar feeds  
✅ **Automatic Merging**: All calendars are merged and sorted chronologically  
✅ **Duplicate Detection**: Automatically removes duplicate events  
✅ **Fault Tolerance**: If one calendar fails to load, others still work  

## Supported ICS Properties

The Calendar Monitor reads these ICS properties:
- **SUMMARY**: Event title
- **DTSTART**: Start date/time
- **DTEND**: End date/time  
- **DESCRIPTION**: Event description
- **LOCATION**: Event location

## Datetime Formats Supported

- UTC: `20241225T120000Z`
- Local: `20241225T120000`
- Date only: `20241225`

## Example ICS File Structure

```ics
BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//Example Corp//Example Calendar//EN
BEGIN:VEVENT
UID:12345@example.com
DTSTART:20241225T120000Z
DTEND:20241225T130000Z
SUMMARY:Team Meeting
DESCRIPTION:Weekly team sync
LOCATION:Conference Room A
END:VEVENT
END:VCALENDAR
```

## Troubleshooting

### Common Issues

1. **"ICS file not found"**: Check the file path in your `calendar-monitor.toml` or environment variables
2. **"No events showing"**: Ensure your ICS file contains events for today
3. **"Parse errors"**: Check that your ICS file is valid (try opening in a calendar app first)

### Fallback Mode

If the ICS file can't be read, the application automatically falls back to mock data so you can still test the interface.

## Security Notes

- ICS files may contain sensitive meeting information
- Be careful when using public ICS URLs
- Consider adding your ICS files to `.gitignore` if they contain private data
y 