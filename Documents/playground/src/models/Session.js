const pool = require('../config/database');
const { v4: uuidv4 } = require('uuid');

class Session {
  /**
   * Create a new session for a user
   * @param {number} userId - User ID
   * @param {string} deviceInfo - Device information
   * @param {string} browserInfo - Browser information
   * @param {string} ipAddress - IP address
   * @param {number} expiresInDays - Session expiry in days (default: 30)
   * @returns {Object} Created session
   */
  static async create(userId, deviceInfo, browserInfo, ipAddress, expiresInDays = 30) {
    const sessionToken = uuidv4();
    const expiresAt = new Date();
    expiresAt.setDate(expiresAt.getDate() + expiresInDays);

    const query = `
      INSERT INTO sessions (user_id, session_token, device_info, browser_info, ip_address, expires_at)
      VALUES ($1, $2, $3, $4, $5, $6)
      RETURNING id, user_id, session_token, device_info, browser_info, ip_address, 
                last_active, created_at, expires_at, is_active
    `;

    const values = [userId, sessionToken, deviceInfo, browserInfo, ipAddress, expiresAt];
    const result = await pool.query(query, values);
    return result.rows[0];
  }

  /**
   * Get all active sessions for a user
   * @param {number} userId - User ID
   * @returns {Array} List of active sessions
   */
  static async getUserSessions(userId) {
    const query = `
      SELECT id, session_token, device_info, browser_info, ip_address, 
             last_active, created_at, expires_at, is_active
      FROM sessions
      WHERE user_id = $1 AND is_active = true AND expires_at > CURRENT_TIMESTAMP
      ORDER BY last_active DESC
    `;

    const result = await pool.query(query, [userId]);
    return result.rows;
  }

  /**
   * Get session by token
   * @param {string} sessionToken - Session token
   * @returns {Object|null} Session object or null
   */
  static async getByToken(sessionToken) {
    const query = `
      SELECT id, user_id, session_token, device_info, browser_info, ip_address, 
             last_active, created_at, expires_at, is_active
      FROM sessions
      WHERE session_token = $1 AND is_active = true AND expires_at > CURRENT_TIMESTAMP
    `;

    const result = await pool.query(query, [sessionToken]);
    return result.rows[0] || null;
  }

  /**
   * Update session last active timestamp
   * @param {string} sessionToken - Session token
   * @returns {boolean} Success status
   */
  static async updateLastActive(sessionToken) {
    const query = `
      UPDATE sessions
      SET last_active = CURRENT_TIMESTAMP
      WHERE session_token = $1 AND is_active = true
      RETURNING id
    `;

    const result = await pool.query(query, [sessionToken]);
    return result.rowCount > 0;
  }

  /**
   * Revoke a specific session
   * @param {number} sessionId - Session ID
   * @param {number} userId - User ID (for verification)
   * @returns {boolean} Success status
   */
  static async revokeSession(sessionId, userId) {
    const query = `
      UPDATE sessions
      SET is_active = false
      WHERE id = $1 AND user_id = $2
      RETURNING id
    `;

    const result = await pool.query(query, [sessionId, userId]);
    return result.rowCount > 0;
  }

  /**
   * Revoke all sessions for a user except the current one
   * @param {number} userId - User ID
   * @param {string} currentSessionToken - Current session token to keep active
   * @returns {number} Number of sessions revoked
   */
  static async revokeAllExceptCurrent(userId, currentSessionToken) {
    const query = `
      UPDATE sessions
      SET is_active = false
      WHERE user_id = $1 AND session_token != $2 AND is_active = true
      RETURNING id
    `;

    const result = await pool.query(query, [userId, currentSessionToken]);
    return result.rowCount;
  }

  /**
   * Revoke all sessions for a user (including current)
   * @param {number} userId - User ID
   * @returns {number} Number of sessions revoked
   */
  static async revokeAllSessions(userId) {
    const query = `
      UPDATE sessions
      SET is_active = false
      WHERE user_id = $1 AND is_active = true
      RETURNING id
    `;

    const result = await pool.query(query, [userId]);
    return result.rowCount;
  }

  /**
   * Delete expired sessions from database
   * @returns {number} Number of sessions deleted
   */
  static async cleanupExpired() {
    const query = `
      DELETE FROM sessions
      WHERE expires_at < CURRENT_TIMESTAMP OR (is_active = false AND last_active < CURRENT_TIMESTAMP - INTERVAL '30 days')
      RETURNING id
    `;

    const result = await pool.query(query);
    return result.rowCount;
  }

  /**
   * Get session count for a user
   * @param {number} userId - User ID
   * @returns {number} Active session count
   */
  static async getActiveSessionCount(userId) {
    const query = `
      SELECT COUNT(*) as count
      FROM sessions
      WHERE user_id = $1 AND is_active = true AND expires_at > CURRENT_TIMESTAMP
    `;

    const result = await pool.query(query, [userId]);
    return parseInt(result.rows[0].count, 10);
  }
}

module.exports = Session;
