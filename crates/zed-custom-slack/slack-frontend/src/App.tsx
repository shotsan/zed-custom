import { useState } from 'react';
import { Routes, Route, Navigate } from 'react-router-dom';
import Auth from './Auth';
import ChatLayout from './ChatLayout';

function App() {
  const [token, setToken] = useState<string | null>(localStorage.getItem('token'));

  return (
    <Routes>
      <Route 
        path="/auth" 
        element={!token ? <Auth setToken={setToken} /> : <Navigate to="/" />} 
      />
      <Route 
        path="/*" 
        element={token ? <ChatLayout setToken={setToken} /> : <Navigate to="/auth" />} 
      />
    </Routes>
  );
}

export default App;
