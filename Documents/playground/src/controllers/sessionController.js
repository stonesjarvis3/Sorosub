const Session = require('../models/Session');

/**
 * Get all active sessions for the authenticated user
 */
const getUserSessions = async (req, res) => {
  try {
    const sessions = await Session.getUserSessions(req.userId);

    // Format sessions for frontend display
    const formattedSessions = sessions.map(session => ({
      id: session.id,
      device: session.device_info,
      browser: session.browser_info,
      ipAddress: session.ip_address,
      lastActive: session.last_active,
      createdAt: session.created_at,
      isCurrent: session.session_token === req.sessionToken
    }));

    res.json({
      success: true,
      sessions: formattedSessions,
      totalCount: formattedSessions.length
    });
  } catch (error) {
    console.error('Error fetching user sessions:', error);
    res.status(500).json({ 
      success: false, 
      error: 'Failed to fetch sessions' 
    });
  }
};

/**
 * Revoke a specific session
 */
const revokeSession = async (req, res) => {
  try {
    const { sessionId } = req.params;

    if (!sessionId) {
      return res.status(400).json({ 
        success: false, 
        error: 'Session ID is required' 
      });
    }

    const success = await Session.revokeSession(parseInt(sessionId, 10), req.userId);

    if (!success) {
      return res.status(404).json({ 
        success: false, 
        error: 'Session not found or already revoked' 
      });
    }

    res.json({
      success: true,
      message: 'Session revoked successfully'
    });
  } catch (error) {
    console.error('Error revoking session:', error);
    res.status(500).json({ 
      success: false, 
      error: 'Failed to revoke session' 
    });
  }
};

/**
 * Sign out from all devices except current
 */
const signOutOtherDevices = async (req, res) => {
  try {
    const revokedCount = await Session.revokeAllExceptCurrent(
      req.userId,
      req.sessionToken
    );

    res.json({
      success: true,
      message: `Signed out from ${revokedCount} device(s)`,
      revokedCount
    });
  } catch (error) {
    console.error('Error signing out other devices:', error);
    res.status(500).json({ 
      success: false, 
      error: 'Failed to sign out other devices' 
    });
  }
};

/**
 * Sign out from all devices (including current)
 */
const signOutAllDevices = async (req, res) => {
  try {
    const revokedCount = await Session.revokeAllSessions(req.userId);

    res.json({
      success: true,
      message: `Signed out from all ${revokedCount} device(s)`,
      revokedCount
    });
  } catch (error) {
    console.error('Error signing out all devices:', error);
    res.status(500).json({ 
      success: false, 
      error: 'Failed to sign out all devices' 
    });
  }
};

/**
 * Get session statistics
 */
const getSessionStats = async (req, res) => {
  try {
    const activeCount = await Session.getActiveSessionCount(req.userId);

    res.json({
      success: true,
      stats: {
        activeSessionCount: activeCount
      }
    });
  } catch (error) {
    console.error('Error fetching session stats:', error);
    res.status(500).json({ 
      success: false, 
      error: 'Failed to fetch session statistics' 
    });
  }
};

module.exports = {
  getUserSessions,
  revokeSession,
  signOutOtherDevices,
  signOutAllDevices,
  getSessionStats
};
