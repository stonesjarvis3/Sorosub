import React, { useState, useEffect } from 'react';
import './SessionList.css';

const SessionList = () => {
  const [sessions, setSessions] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    fetchSessions();
  }, []);

  const fetchSessions = async () => {
    try {
      setLoading(true);
      const response = await fetch('/api/sessions', {
        headers: {
          'x-session-token': localStorage.getItem('sessionToken')
        },
        credentials: 'include'
      });

      if (!response.ok) {
        throw new Error('Failed to fetch sessions');
      }

      const data = await response.json();
      setSessions(data.sessions);
      setError(null);
    } catch (err) {
      setError(err.message);
      console.error('Error fetching sessions:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleRevokeSession = async (sessionId) => {
    if (!window.confirm('Are you sure you want to revoke this session?')) {
      return;
    }

    try {
      const response = await fetch(`/api/sessions/${sessionId}`, {
        method: 'DELETE',
        headers: {
          'x-session-token': localStorage.getItem('sessionToken')
        },
        credentials: 'include'
      });

      if (!response.ok) {
        throw new Error('Failed to revoke session');
      }

      // Refresh session list
      await fetchSessions();
    } catch (err) {
      setError(err.message);
      console.error('Error revoking session:', err);
    }
  };

  const handleSignOutOthers = async () => {
    if (!window.confirm('Sign out from all other devices?')) {
      return;
    }

    try {
      const response = await fetch('/api/sessions/sign-out-others', {
        method: 'POST',
        headers: {
          'x-session-token': localStorage.getItem('sessionToken')
        },
        credentials: 'include'
      });

      if (!response.ok) {
        throw new Error('Failed to sign out from other devices');
      }

      const data = await response.json();
      alert(data.message);
      await fetchSessions();
    } catch (err) {
      setError(err.message);
      console.error('Error signing out from other devices:', err);
    }
  };

  const handleSignOutAll = async () => {
    if (!window.confirm('Sign out from ALL devices? You will need to log in again.')) {
      return;
    }

    try {
      const response = await fetch('/api/sessions/sign-out-all', {
        method: 'POST',
        headers: {
          'x-session-token': localStorage.getItem('sessionToken')
        },
        credentials: 'include'
      });

      if (!response.ok) {
        throw new Error('Failed to sign out from all devices');
      }

      // Clear local session and redirect to login
      localStorage.removeItem('sessionToken');
      window.location.href = '/login';
    } catch (err) {
      setError(err.message);
      console.error('Error signing out from all devices:', err);
    }
  };

  const formatLastActive = (timestamp) => {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now - date;
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins} minute${diffMins > 1 ? 's' : ''} ago`;
    if (diffHours < 24) return `${diffHours} hour${diffHours > 1 ? 's' : ''} ago`;
    return `${diffDays} day${diffDays > 1 ? 's' : ''} ago`;
  };

  if (loading) {
    return <div className="session-list-loading">Loading sessions...</div>;
  }

  if (error) {
    return <div className="session-list-error">Error: {error}</div>;
  }

  return (
    <div className="session-list-container">
      <div className="session-list-header">
        <h2>Active Sessions</h2>
        <p className="session-count">{sessions.length} active session{sessions.length !== 1 ? 's' : ''}</p>
      </div>

      {sessions.length === 0 ? (
        <div className="no-sessions">No active sessions found</div>
      ) : (
        <div className="sessions">
          {sessions.map((session) => (
            <div 
              key={session.id} 
              className={`session-card ${session.isCurrent ? 'current-session' : ''}`}
            >
              <div className="session-info">
                <div className="session-device">
                  <span className="device-icon">
                    {session.device === 'Mobile' ? '📱' : 
                     session.device === 'Tablet' ? '💻' : '🖥️'}
                  </span>
                  <div className="device-details">
                    <strong>{session.browser}</strong> on {session.device}
                    {session.isCurrent && <span className="current-badge">Current</span>}
                  </div>
                </div>
                
                <div className="session-meta">
                  <div className="session-ip">
                    <span className="meta-label">IP Address:</span> {session.ipAddress}
                  </div>
                  <div className="session-last-active">
                    <span className="meta-label">Last active:</span> {formatLastActive(session.lastActive)}
                  </div>
                </div>
              </div>

              {!session.isCurrent && (
                <button 
                  className="revoke-button"
                  onClick={() => handleRevokeSession(session.id)}
                >
                  Sign Out
                </button>
              )}
            </div>
          ))}
        </div>
      )}

      <div className="session-actions">
        <button 
          className="btn-secondary"
          onClick={handleSignOutOthers}
          disabled={sessions.filter(s => !s.isCurrent).length === 0}
        >
          Sign Out Other Devices
        </button>
        <button 
          className="btn-danger"
          onClick={handleSignOutAll}
        >
          Sign Out All Devices
        </button>
      </div>
    </div>
  );
};

export default SessionList;
