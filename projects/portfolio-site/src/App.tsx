import { useState } from 'react'
import './App.css'

function App() {
  const [items, setItems] = useState([
    'Visual UI generated automatically',
    'Modern, responsive design',
    'Ready for customization'
  ])
  const [newItem, setNewItem] = useState('')

  const addItem = () => {
    if (newItem.trim()) {
      setItems([...items, newItem])
      setNewItem('')
    }
  }

  return (
    <div className="app">
      <div className="header">
        <h1>portfolio-site</h1>
        <p>Personal portfolio website</p>
      </div>

      <div className="content">
        <h2>Getting Started</h2>
        <p style={{ color: '#718096', marginBottom: '24px' }}>
          Your project has been set up with a beautiful visual interface. Start customizing!
        </p>

        <div>
          <h3>Features</h3>
          {items.map((item, index) => (
            <div key={index} className="card">
              ✓ {item}
            </div>
          ))}
        </div>

        <div style={{ marginTop: '24px', display: 'flex', gap: '12px' }}>
          <input
            type="text"
            value={newItem}
            onChange={(e) => setNewItem(e.target.value)}
            onKeyPress={(e) => e.key === 'Enter' && addItem()}
            placeholder="Add a feature..."
            style={{
              flex: 1,
              padding: '12px',
              borderRadius: '8px',
              border: '1px solid #e2e8f0',
              fontSize: '16px'
            }}
          />
          <button className="button" onClick={addItem}>Add</button>
        </div>
      </div>
    </div>
  )
}

export default App
