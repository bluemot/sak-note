import { useState, useCallback, useRef, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/tauri'
import './LlmChat.css'

interface Message {
  role: 'user' | 'assistant' | 'system'
  content: string
  timestamp: number
}

interface LlmResponse {
  response: string
  model: string
  done: boolean
}

interface LlmChatProps {
  filePath: string
}

export default function LlmChat({ filePath }: LlmChatProps) {
  const [messages, setMessages] = useState<Message[]>([])
  const [input, setInput] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [model, setModel] = useState('kimi-k2.5:cloud')
  const [isConnected, setIsConnected] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const messagesEndRef = useRef<HTMLDivElement>(null)
  
  const contextId = filePath || 'default'
  
  // Check connection on mount
  useEffect(() => {
    checkConnection()
  }, [])
  
  // Scroll to bottom on new messages
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages])
  
  const checkConnection = async () => {
    try {
      await invoke<{ models: Array<{ name: string }> }>('execute_module', {
        module: 'llm',
        capability: 'list_models',
        input: { api_url: 'https://ollama.com' }
      })
      setIsConnected(true)
      setError(null)
    } catch (err) {
      setIsConnected(false)
      setError(err instanceof Error ? err.message : String(err))
    }
  }
  
  const sendMessage = useCallback(async () => {
    if (!input.trim() || isLoading) return
    
    const userMessage: Message = {
      role: 'user',
      content: input.trim(),
      timestamp: Date.now()
    }
    
    setMessages(prev => [...prev, userMessage])
    setInput('')
    setIsLoading(true)
    setError(null)
    
    try {
      const response = await invoke<LlmResponse>('execute_module', {
        module: 'llm',
        capability: 'chat',
        input: {
          message: userMessage.content,
          context_id: contextId,
          model: model
        }
      })
      
      const assistantMessage: Message = {
        role: 'assistant',
        content: response.response,
        timestamp: Date.now()
      }
      
      setMessages(prev => [...prev, assistantMessage])
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setIsLoading(false)
    }
  }, [input, isLoading, contextId, model])
  
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      sendMessage()
    }
  }
  
  const clearChat = async () => {
    try {
      await invoke('execute_module', {
        module: 'llm',
        capability: 'clear_context',
        input: { context_id: contextId }
      })
      setMessages([])
    } catch (err) {
      console.error('Failed to clear context:', err)
    }
  }
  
  return (
    <div className="llm-chat">
      <div className="chat-header">
        <h3>💬 LLM Chat</h3>
        <div className="chat-status">
          <span className={`status-dot ${isConnected ? 'connected' : 'disconnected'}`} />
          <span>{isConnected ? 'Connected' : 'Disconnected'}</span>
        </div>
      </div>
      
      <div className="model-selector">
        <label>Model:</label>
        <select value={model} onChange={(e) => setModel(e.target.value)}>
          <option value="kimi-k2.5:cloud">kimi-k2.5:cloud</option>
          <option value="llama3">llama3</option>
          <option value="qwen2.5:14b">qwen2.5:14b</option>
        </select>
      </div>
      
      <div className="chat-messages">
        {messages.length === 0 && (
          <div className="chat-empty">
            <p>Ask me anything about your file!</p>
            <p className="hint">Press Enter to send, Shift+Enter for new line</p>
          </div>
        )}
        
        {messages.map((msg, idx) => (
          <div key={idx} className={`message message-${msg.role}`}>
            <div className="message-role">
              {msg.role === 'user' ? '👤' : msg.role === 'assistant' ? '🤖' : '⚙️'}
            </div>
            <div className="message-content">
              <pre>{msg.content}</pre>
            </div>
          </div>
        ))}
        
        {isLoading && (
          <div className="message message-assistant loading">
            <div className="message-role">🤖</div>
            <div className="message-content">
              <span className="loading-dots">Thinking...</span>
            </div>
          </div>
        )}
        
        {error && (
          <div className="chat-error">
            Error: {error}
          </div>
        )}
        
        <div ref={messagesEndRef} />
      </div>
      
      <div className="chat-input-container">
        <textarea
          className="chat-input"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Ask about the file..."
          rows={2}
          disabled={isLoading}
        />
        <div className="chat-actions">
          <button onClick={clearChat} className="clear-btn" title="Clear chat">
            🗑️
          </button>
          <button onClick={sendMessage} className="send-btn" disabled={!input.trim() || isLoading}>
            {isLoading ? '⏳' : '📤'}
          </button>
        </div>
      </div>
    </div>
  )
}
