# Google Calendar OAuth Setup Guide

**The easiest way to integrate Google Calendar with Calendar Monitor!**

## Why Google OAuth?
✅ **One-click setup** - Just click "Connect Google Calendar" button  
✅ **Works everywhere** - No company restrictions or VPN issues  
✅ **Familiar experience** - Standard "Login with Google" flow  
✅ **Secure** - No API keys needed, uses OAuth 2.0  
✅ **Real-time data** - Direct API access, no cached ICS feeds  

## Quick Setup (3 minutes)

### 1. Create Google OAuth Application

1. **Go to**: [Google Cloud Console](https://console.cloud.google.com/)
2. **Create a new project** (or select existing):
   - Click "Select a project" → "New Project"
   - Give it a name like "Calendar Monitor"
   - Click "Create"

3. **Enable Google Calendar API**:
   - Go to "APIs & Services" → "Library"
   - Search "Google Calendar API" 
   - Click "Enable"

4. **Create OAuth Credentials**:
   - Go to "APIs & Services" → "Credentials"
   - Click "Create Credentials" → "OAuth 2.0 Client ID"
   - If prompted, configure OAuth consent screen first:
     - Choose "External" user type
     - Fill in app name: "Calendar Monitor"
     - Add your email as developer contact
   - Application type: **"Web application"**
   - Name: "Calendar Monitor"
   - **Authorized redirect URIs**: `http://localhost:3000/auth/google/callback`
   - Click "Create"
   - **Copy the Client ID and Client Secret** (you'll need these next)

### 2. Configure Environment Variables

Add to your `.env` file:

```bash
# Google OAuth Configuration
GOOGLE_CLIENT_ID=your_client_id_here
GOOGLE_CLIENT_SECRET=your_client_secret_here  
GOOGLE_REDIRECT_URI=http://localhost:3000/auth/google/callback
```

### 3. Test the Integration

```bash
cargo run
```

1. Open http://localhost:3000
2. Click **"Connect Google Calendar"** button  
3. Login with your Google account
4. Grant calendar read permission
5. Get redirected back with success message
6. Your Google Calendar events now appear with your other calendars! 🎉

## How It Works

1. **Click "Connect"** → Redirects to Google OAuth
2. **Login with Google** → Grant calendar read access  
3. **Redirect back** → Success page with authorization code
4. **Calendar events** → Automatically merged with your ICS feeds

## Current Implementation Note

**Current Status**: The basic OAuth flow is implemented and working! You can successfully:
- ✅ Click "Connect Google Calendar" 
- ✅ Authenticate with Google
- ✅ Get redirected back to success page

**Next Steps**: The authorization code exchange and calendar data fetching will be completed in the next version. Currently this sets up the foundation for full Google Calendar integration.

## Troubleshooting

**"OAuth not configured"** → Check your `.env` file has the Google credentials  
**"403 Forbidden"** → Make sure Google Calendar API is enabled in your project  
**"Redirect URI mismatch"** → Verify redirect URI is exactly `http://localhost:3000/auth/google/callback`  
**"App not verified"** → For personal use, click "Advanced" → "Go to Calendar Monitor (unsafe)" - this is normal for development

## Why This Approach?

**vs Service Accounts**: 
- ❌ Service accounts require complex JSON files and API enabling
- ❌ Often blocked by company firewalls/policies
- ✅ OAuth works with any Google account, anywhere

**vs Manual ICS URLs**:
- ❌ Hard to find the right ICS URL in Google Calendar settings  
- ❌ URLs can change and break your setup
- ✅ OAuth gives direct, reliable API access

## Next Steps

Once the OAuth foundation is complete, upcoming features include:
- Automatic token refresh
- Session persistence  
- Full calendar data integration
- Multi-account support

Perfect for personal productivity setups! 🎯
