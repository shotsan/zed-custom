import { useEffect, useState, useRef } from 'react';
import { Centrifuge } from 'centrifuge';
import { Send, Hash, Server } from 'lucide-react';
import './index.css';

interface ChatMessage {
  id: string;
  user: string;
  text: string;
  isIncoming: boolean;
  timestamp: string;
}

function App() {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [inputText, setInputText] = useState('');
  const [isConnected, setIsConnected] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    // 1. Initialize Centrifugo Client
    const centrifuge = new Centrifuge('ws://localhost:8000/connection/websocket', {
      // In production, you would fetch a JWT from your Axum backend and pass it here.
      // For this local prototype, we enabled anonymous access in centrifugo_config.json
    });

    centrifuge.on('connected', function (ctx) {
      console.log('Connected to Centrifugo', ctx);
      setIsConnected(true);
    });

    centrifuge.on('disconnected', function (ctx) {
      console.log('Disconnected from Centrifugo', ctx);
      setIsConnected(false);
    });

    // 2. Subscribe to the Slack Channel
    const sub = centrifuge.newSubscription('slack:general');

    sub.on('publication', function (ctx) {
      const payload = ctx.data;
      const newMessage: ChatMessage = {
        id: Math.random().toString(36).substring(7),
        user: payload.user || 'Unknown User',
        text: payload.text,
        isIncoming: true, // Messages from Centrifugo are coming from Slack
        timestamp: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
      };
      setMessages((prev) => [...prev, newMessage]);
    });

    sub.on('subscribed', function (ctx) {
      console.log('Subscribed to slack:general', ctx);
    });

    sub.subscribe();
    centrifuge.connect();

    return () => {
      sub.unsubscribe();
      centrifuge.disconnect();
    };
  }, []);

  useEffect(() => {
    // Auto-scroll to bottom when new messages arrive
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  const handleSendMessage = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!inputText.trim()) return;

    // Optimistically add the message to the UI
    const outgoingMsg: ChatMessage = {
      id: Math.random().toString(36).substring(7),
      user: 'You (Web Client)',
      text: inputText,
      isIncoming: false,
      timestamp: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
    };
    setMessages((prev) => [...prev, outgoingMsg]);
    setInputText('');

    // POST to our local Axum Proxy
    try {
      await fetch('http://localhost:3001/api/messages', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          channel: 'general',
          text: outgoingMsg.text,
        }),
      });
    } catch (err) {
      console.error('Failed to send message to Axum proxy', err);
    }
  };

  return (
    <div className="app-container">
      {/* Header */}
      <header className="header">
        <div className="header-title">
          <Hash size={24} color="var(--accent-primary)" />
          <span>general</span>
        </div>
        <div className="header-status">
          <Server size={16} />
          <span>{isConnected ? 'Connected' : 'Reconnecting...'}</span>
          <div className={`status-indicator ${isConnected ? '' : 'disconnected'}`}></div>
        </div>
      </header>

      {/* Chat Area */}
      <main className="chat-container">
        {messages.length === 0 ? (
          <div style={{ margin: 'auto', color: 'var(--text-secondary)' }}>
            No messages yet. Send a webhook to the Axum proxy to see it appear here!
          </div>
        ) : (
          messages.map((msg) => (
            <div key={msg.id} className={`message ${msg.isIncoming ? 'incoming' : 'outgoing'}`}>
              <div className="message-meta">
                <span className="message-user">{msg.user}</span>
                <span>{msg.timestamp}</span>
              </div>
              <div className="message-bubble">{msg.text}</div>
            </div>
          ))
        )}
        <div ref={messagesEndRef} />
      </main>

      {/* Input Area */}
      <form className="input-area" onSubmit={handleSendMessage}>
        <input
          type="text"
          className="chat-input"
          placeholder="Message #general..."
          value={inputText}
          onChange={(e) => setInputText(e.target.value)}
        />
        <button type="submit" className="send-button" disabled={!inputText.trim()}>
          <Send size={20} />
        </button>
      </form>
    </div>
  );
}

export default App;
