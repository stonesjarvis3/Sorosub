const express = require('express');
const router = express.Router();
const { authenticateSession } = require('../middleware/auth');
const {
  getUserSessions,
  revokeSession,
  signOutOtherDevices,
  signOutAllDevices,
  getSessionStats
} = require('../controllers/sessionController');

// All session routes require authentication
router.use(authenticateSession);

// Get all active sessions for the user
router.get('/sessions', getUserSessions);

// Get session statistics
router.get('/sessions/stats', getSessionStats);

// Revoke a specific session
router.delete('/sessions/:sessionId', revokeSession);

// Sign out from all other devices (except current)
router.post('/sessions/sign-out-others', signOutOtherDevices);

// Sign out from all devices (including current)
router.post('/sessions/sign-out-all', signOutAllDevices);

module.exports = router;
