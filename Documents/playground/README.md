# Session Management Feature

## Overview

This implementation provides comprehensive session tracking and management functionality for users, addressing issue #337.

## Features

### ✅ Acceptance Criteria Met

1. **Active Sessions Stored in Database**
   - Sessions stored with device info, browser info, IP address, and last active timestamp
   - Automatic tracking of session metadata

2. **Session List in Profile Settings**
   - Complete UI component displaying all active sessions
   - Real-time session information with device icons
   - Visual indication of current session

3. **Individual Session Revocation**
   - Users can revoke any specific session
   - "Sign Out" button for each session (except current)

4. **'Sign Out All' Button**
   - Sign out from all other devices
   - Sign out from all devices (including current)

## Architecture

### Backend Components

- **Database Schema** (`database/schema.sql`)
  - `sessions` table with comprehensive tracking
  - Indexes for optimal query performance
  - Automatic cleanup triggers

- **Session Model** (`src/models/Session.js`)
  - CRUD operations for sessions
  - Session validation and expiry management
  - Batch operations for revocation

- **Authentication Middleware** (`src/middleware/auth.js`)
  - Session token validation
  - Device and browser detection
  - IP address extraction

- **Session Controller** (`src/controllers/sessionController.js`)
  - RESTful API endpoints
  - Business logic for session operations

- **Routes** (`src/routes/sessionRoutes.js`)
  - `/api/sessions` - Get all active sessions
  - `/api/sessions/stats` - Get session statistics
  - `/api/sessions/:sessionId` - Revoke specific session
  - `/api/sessions/sign-out-others` - Sign out other devices
  - `/api/sessions/sign-out-all` - Sign out all devices

### Frontend Components

- **SessionList Component** (`frontend/components/SessionList.jsx`)
  - Display active sessions with device info
  - Session revocation functionality
  - Sign out actions

- **Profile Settings Page** (`frontend/pages/ProfileSettings.jsx`)
  - Tabbed interface for user settings
  - Integration of SessionList component

## Installation

1. **Install Dependencies**
   ```bash
   npm install
   ```

2. **Setup Database**
   ```bash
   # Create PostgreSQL database
   createdb session_management_db
   
   # Run schema
   psql -d session_management_db -f database/schema.sql
   ```

3. **Configure Environment**
   ```bash
   cp .env.example .env
   # Edit .env with your database credentials
   ```

4. **Start Server**
   ```bash
   # Development
   npm run dev
   
   # Production
   npm start
   ```

## API Documentation

### Get Active Sessions
```http
GET /api/sessions
Headers: x-session-token: <token>

Response: {
  success: true,
  sessions: [
    {
      id: 1,
      device: "Desktop",
      browser: "Chrome",
      ipAddress: "192.168.1.1",
      lastActive: "2026-06-02T10:30:00Z",
      isCurrent: true
    }
  ],
  totalCount: 1
}
```

### Revoke Specific Session
```http
DELETE /api/sessions/:sessionId
Headers: x-session-token: <token>

Response: {
  success: true,
  message: "Session revoked successfully"
}
```

### Sign Out Other Devices
```http
POST /api/sessions/sign-out-others
Headers: x-session-token: <token>

Response: {
  success: true,
  message: "Signed out from 3 device(s)",
  revokedCount: 3
}
```

### Sign Out All Devices
```http
POST /api/sessions/sign-out-all
Headers: x-session-token: <token>

Response: {
  success: true,
  message: "Signed out from all 4 device(s)",
  revokedCount: 4
}
```

## Security Features

- Session token-based authentication
- IP address tracking for suspicious activity detection
- Automatic cleanup of expired sessions
- Device fingerprinting (browser and device type)
- Secure session revocation

## Database Schema

```sql
sessions {
  id              SERIAL PRIMARY KEY
  user_id         INTEGER (FK to users)
  session_token   VARCHAR(255) UNIQUE
  device_info     VARCHAR(500)
  browser_info    VARCHAR(500)
  ip_address      INET
  last_active     TIMESTAMP
  created_at      TIMESTAMP
  expires_at      TIMESTAMP
  is_active       BOOLEAN
}
```

## Frontend Integration

```jsx
import SessionList from './components/SessionList';

function ProfileSettings() {
  return (
    <div>
      <h1>Profile Settings</h1>
      <SessionList />
    </div>
  );
}
```

## Testing

The implementation includes:
- Automatic session expiry (30 days default)
- Hourly cleanup of expired sessions
- Last active timestamp updates on each request
- Session validation on every API call

## Future Enhancements

- Email notifications on new device login
- Suspicious activity detection
- Session analytics dashboard
- Geographic location tracking
- Mobile app session management

## Issue Reference

Closes #337 - Session Management and Device Sign Out

## License

MIT
