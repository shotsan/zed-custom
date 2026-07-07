import { useState } from 'react';
import { Box, Button, TextField, Typography, Paper, Alert } from '@mui/material';
import axios, { AxiosError } from 'axios';

const API_BASE = 'http://127.0.0.1:8080';

interface AuthProps {
  setToken: (token: string) => void;
}

export default function Auth({ setToken }: AuthProps) {
  const [email, setEmail] = useState('');
  const [code, setCode] = useState('');
  const [step, setStep] = useState<'LOGIN' | 'VERIFY'>('LOGIN');
  const [error, setError] = useState('');

  const handleLogin = async () => {
    try {
      setError('');
      await axios.post(`${API_BASE}/auth/login`, { email });
      setStep('VERIFY');
    } catch (err) {
      if (err instanceof AxiosError) {
        setError(err.response?.data?.message || err.message);
      } else {
        setError(String(err));
      }
    }
  };

  const handleVerify = async () => {
    try {
      setError('');
      const res = await axios.post(`${API_BASE}/auth/verify`, { email, code });
      const token = res.data.token;
      localStorage.setItem('token', token);
      setToken(token);
    } catch (err) {
      if (err instanceof AxiosError) {
        setError(err.response?.data?.message || err.message);
      } else {
        setError(String(err));
      }
    }
  };

  return (
    <Box sx={{
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      minHeight: '100vh',
      bgcolor: 'background.default'
    }}>
      <Paper elevation={4} sx={{ p: 4, width: '100%', maxWidth: 400, borderRadius: 3 }}>
        <Typography variant="h4" gutterBottom color="primary" sx={{ fontWeight: 'bold' }}>
          Slack Login
        </Typography>
        <Typography variant="body1" color="text.secondary" sx={{ mb: 3 }}>
          {step === 'LOGIN' ? 'Enter your email to receive a 2FA code.' : 'Enter the code sent to your email.'}
        </Typography>

        {error && <Alert severity="error" sx={{ mb: 2 }}>{error}</Alert>}

        {step === 'LOGIN' ? (
          <>
            <TextField
              fullWidth
              label="Email Address"
              variant="outlined"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              sx={{ mb: 3 }}
            />
            <Button 
              fullWidth 
              variant="contained" 
              size="large" 
              onClick={handleLogin}
              disabled={!email}
            >
              Send Code
            </Button>
          </>
        ) : (
          <>
            <TextField
              fullWidth
              label="2FA Code"
              variant="outlined"
              value={code}
              onChange={(e) => setCode(e.target.value)}
              sx={{ mb: 3 }}
            />
            <Button 
              fullWidth 
              variant="contained" 
              size="large" 
              onClick={handleVerify}
              disabled={!code}
            >
              Verify & Login
            </Button>
          </>
        )}
      </Paper>
    </Box>
  );
}
