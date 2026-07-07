import { useEffect, useState } from 'react';
import { 
  Box, 
  Drawer, 
  List, 
  ListItem, 
  ListItemButton, 
  ListItemIcon, 
  ListItemText, 
  Typography, 
  Divider,
  AppBar,
  Toolbar,
  IconButton,
  TextField,
  Paper,
  Avatar
} from '@mui/material';
import { 
  Hash, 
  LogOut, 
  Send
} from 'lucide-react';
import axios from 'axios';

const drawerWidth = 260;
const API_BASE = 'http://127.0.0.1:8080';

interface ChatLayoutProps {
  setToken: (t: string | null) => void;
}

interface Message {
  id: string;
  user: string;
  text: string;
  timestamp: string;
}

export default function ChatLayout({ setToken }: ChatLayoutProps) {
  const [messages, setMessages] = useState<Message[]>([]);
  const [inputText, setInputText] = useState('');

  useEffect(() => {
    // Fetch initial messages using the token
    const token = localStorage.getItem('token');
    axios.get(`${API_BASE}/messages`, {
      headers: { Authorization: `Bearer ${token}` }
    })
    .then(res => setMessages(res.data))
    .catch(() => {
      // If unauthorized, log out
      handleLogout();
    });
  }, []);

  const handleLogout = () => {
    localStorage.removeItem('token');
    setToken(null);
  };

  const handleSend = () => {
    if (!inputText.trim()) return;
    
    // Simulate sending message locally for now
    const newMsg: Message = {
      id: Date.now().toString(),
      user: 'Me',
      text: inputText,
      timestamp: new Date().toLocaleTimeString(),
    };
    
    setMessages([...messages, newMsg]);
    setInputText('');
  };

  return (
    <Box sx={{ display: 'flex', height: '100vh', overflow: 'hidden' }}>
      {/* Sidebar */}
      <Drawer
        variant="permanent"
        sx={{
          width: drawerWidth,
          flexShrink: 0,
          '& .MuiDrawer-paper': {
            width: drawerWidth,
            boxSizing: 'border-box',
            backgroundColor: 'background.paper',
            borderRight: '1px solid rgba(255,255,255,0.05)'
          },
        }}
      >
        <Box sx={{ p: 2 }}>
          <Typography variant="h6" color="primary" sx={{ fontWeight: 'bold' }}>
            Slack
          </Typography>
        </Box>
        <Divider sx={{ opacity: 0.1 }} />
        
        <List sx={{ flexGrow: 1 }}>
          <ListItem disablePadding>
            <ListItemButton selected>
              <ListItemIcon sx={{ minWidth: 40 }}>
                <Hash size={20} />
              </ListItemIcon>
              <ListItemText primary="general" />
            </ListItemButton>
          </ListItem>
          <ListItem disablePadding>
            <ListItemButton>
              <ListItemIcon sx={{ minWidth: 40 }}>
                <Hash size={20} />
              </ListItemIcon>
              <ListItemText primary="random" />
            </ListItemButton>
          </ListItem>
        </List>
        
        <Divider sx={{ opacity: 0.1 }} />
        <List>
          <ListItem disablePadding>
            <ListItemButton onClick={handleLogout}>
              <ListItemIcon sx={{ minWidth: 40 }}>
                <LogOut size={20} color="#F43F5E" />
              </ListItemIcon>
              <ListItemText primary="Logout" sx={{ color: '#F43F5E' }} />
            </ListItemButton>
          </ListItem>
        </List>
      </Drawer>

      {/* Main Content Area */}
      <Box sx={{ flexGrow: 1, display: 'flex', flexDirection: 'column', bgcolor: 'background.default' }}>
        
        {/* Header */}
        <AppBar position="static" color="transparent" elevation={0} sx={{ borderBottom: '1px solid rgba(255,255,255,0.05)' }}>
          <Toolbar>
            <Hash size={24} style={{ marginRight: 8, color: '#4A154B' }} />
            <Typography variant="h6" sx={{ fontWeight: 'bold' }}>
              general
            </Typography>
          </Toolbar>
        </AppBar>

        {/* Message Feed */}
        <Box sx={{ flexGrow: 1, p: 3, overflowY: 'auto' }}>
          {messages.map((msg) => (
            <Box key={msg.id} sx={{ display: 'flex', mb: 3 }}>
              <Avatar sx={{ bgcolor: 'primary.main', mr: 2 }}>
                {msg.user.charAt(0).toUpperCase()}
              </Avatar>
              <Box>
                <Box sx={{ display: 'flex', alignItems: 'baseline', mb: 0.5 }}>
                  <Typography variant="subtitle2" sx={{ mr: 1, fontWeight: 'bold' }}>
                    {msg.user}
                  </Typography>
                  <Typography variant="caption" color="text.secondary">
                    {msg.timestamp}
                  </Typography>
                </Box>
                <Typography variant="body1">
                  {msg.text}
                </Typography>
              </Box>
            </Box>
          ))}
        </Box>

        {/* Message Input Box */}
        <Box sx={{ p: 2 }}>
          <Paper 
            elevation={2} 
            sx={{ 
              p: '2px 4px', 
              display: 'flex', 
              alignItems: 'center', 
              borderRadius: 3,
              bgcolor: 'background.paper'
            }}
          >
            <TextField
              sx={{ ml: 1, flex: 1, '& .MuiInput-underline:before': { borderBottom: 'none' }, '& .MuiInput-underline:after': { borderBottom: 'none' } }}
              placeholder="Message #general"
              variant="standard"
              value={inputText}
              onChange={(e) => setInputText(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') handleSend();
              }}
            />
            <IconButton color="primary" sx={{ p: '10px' }} onClick={handleSend} disabled={!inputText.trim()}>
              <Send size={20} />
            </IconButton>
          </Paper>
        </Box>

      </Box>
    </Box>
  );
}
