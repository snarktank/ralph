import { useState } from 'react'
import './App.css'

function App() {
  const [messages, setMessages] = useState([
    { id: 1, text: 'Welcome to chat-app!', sender: 'system' },
    { id: 2, text: 'Real-time chat application', sender: 'system' }
  ])
  const [input, setInput] = useState('')

  const sendMessage = () => {
    if (input.trim()) {
      setMessages([...messages, { id: Date.now(), text: input, sender: 'user' }])
      setInput('')

      setTimeout(() => {
        setMessages(prev => [...prev, {
          id: Date.now(),
          text: 'This is a demo response!',
          sender: 'bot'
        }])
      }, 1000)
    }
  }

  return (
    <div className="app">
      <div className="header">
        <h1>chat-app</h1>
        <p>Real-time chat application</p>
      </div>

      <div className="content" style={{ height: '600px', display: 'flex', flexDirection: 'column' }}>
        <div style={{ flex: 1, overflowY: 'auto', marginBottom: '20px' }}>
          {messages.map(msg => (
            <div key={msg.id} className="card" style={{
              marginLeft: msg.sender === 'user' ? 'auto' : '0',
              marginRight: msg.sender === 'user' ? '0' : 'auto',
              maxWidth: '70%',
              background: msg.sender === 'user' ? '#4299e1' : '#f7fafc',
              color: msg.sender === 'user' ? 'white' : 'inherit'
            }}>
              {msg.text}
            </div>
          ))}
        </div>

        <div style={{ display: 'flex', gap: '12px' }}>
          <input
            type="text"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyPress={(e) => e.key === 'Enter' && sendMessage()}
            placeholder="Type a message..."
            style={{
              flex: 1,
              padding: '12px',
              borderRadius: '8px',
              border: '1px solid #e2e8f0',
              fontSize: '16px'
            }}
          />
          <button className="button" onClick={sendMessage}>Send</button>
        </div>
      </div>
    </div>
  )
}

export default App
