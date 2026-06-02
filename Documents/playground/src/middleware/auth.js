const Session = require('../models/Session');

/**
 * Middleware to authenticate requests using session token
 */
const authenticateSession = async (req, res, next) => {
  try {
    // Get session token from header
    const sessionToken = req.headers['x-session-token'];

    if (!sessionToken) {
      return res.status(401).json({ error: 'No session token provided' });
    }

    // Verify session
    const session = await Session.getByToken(sessionToken);

    if (!session) {
      return res.status(401).json({ error: 'Invalid or expired session' });
    }

    // Update last active timestamp
    await Session.updateLastActive(sessionToken);

    // Attach session and user info to request
    req.session = session;
    req.userId = session.user_id;
    req.sessionToken = sessionToken;

    next();
  } catch (error) {
    console.error('Authentication error:', error);
    res.status(500).json({ error: 'Authentication failed' });
  }
};

/**
 * Extract device and browser information from request
 */
const extractDeviceInfo = (req) => {
  const userAgent = req.headers['user-agent'] || '';
  
  // Simple device detection
  let device = 'Desktop';
  if (/mobile/i.test(userAgent)) {
    device = 'Mobile';
  } else if (/tablet|ipad/i.test(userAgent)) {
    device = 'Tablet';
  }

  return device;
};

/**
 * Extract browser information from request
 */
const extractBrowserInfo = (req) => {
  const userAgent = req.headers['user-agent'] || '';
  
  // Simple browser detection
  if (/chrome|chromium|crios/i.test(userAgent) && !/edg/i.test(userAgent)) {
    return 'Chrome';
  } else if (/firefox|fxios/i.test(userAgent)) {
    return 'Firefox';
  } else if (/safari/i.test(userAgent) && !/chrome/i.test(userAgent)) {
    return 'Safari';
  } else if (/edg/i.test(userAgent)) {
    return 'Edge';
  } else if (/opr\//i.test(userAgent)) {
    return 'Opera';
  }
  
  return 'Unknown';
};

/**
 * Get client IP address from request
 */
const getClientIp = (req) => {
  return req.headers['x-forwarded-for']?.split(',')[0].trim() ||
         req.headers['x-real-ip'] ||
         req.connection?.remoteAddress ||
         req.socket?.remoteAddress ||
         'Unknown';
};

module.exports = {
  authenticateSession,
  extractDeviceInfo,
  extractBrowserInfo,
  getClientIp
};
