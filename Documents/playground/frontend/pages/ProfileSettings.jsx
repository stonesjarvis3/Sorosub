import React, { useState } from 'react';
import SessionList from '../components/SessionList';
import './ProfileSettings.css';

const ProfileSettings = () => {
  const [activeTab, setActiveTab] = useState('sessions');

  return (
    <div className="profile-settings">
      <div className="settings-sidebar">
        <h1>Settings</h1>
        <nav className="settings-nav">
          <button
            className={`nav-item ${activeTab === 'profile' ? 'active' : ''}`}
            onClick={() => setActiveTab('profile')}
          >
            Profile
          </button>
          <button
            className={`nav-item ${activeTab === 'sessions' ? 'active' : ''}`}
            onClick={() => setActiveTab('sessions')}
          >
            Active Sessions
          </button>
          <button
            className={`nav-item ${activeTab === 'security' ? 'active' : ''}`}
            onClick={() => setActiveTab('security')}
          >
            Security
          </button>
        </nav>
      </div>

      <div className="settings-content">
        {activeTab === 'profile' && (
          <div className="settings-section">
            <h2>Profile Settings</h2>
            <p>Profile settings content goes here...</p>
          </div>
        )}

        {activeTab === 'sessions' && (
          <div className="settings-section">
            <SessionList />
          </div>
        )}

        {activeTab === 'security' && (
          <div className="settings-section">
            <h2>Security Settings</h2>
            <p>Security settings content goes here...</p>
          </div>
        )}
      </div>
    </div>
  );
};

export default ProfileSettings;
