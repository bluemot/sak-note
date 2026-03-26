import { useState } from 'react'
import { useSemantic } from '../hooks/useSemantic'
import './SemanticDemo.css'

export function SemanticDemo() {
  const [filePath, setFilePath] = useState('/home/user/project/src/main.rs')
  const [queryStr, setQueryStr] = useState('function called main')
  const [result, setResult] = useState<any>(null)
  
  const { loading, error, parseFile, query, exportToLLM } = useSemantic()

  const handleParse = async () => {
    const res = await parseFile(filePath)
    setResult(res)
  }

  const handleQuery = async () => {
    const res = await query(filePath, queryStr)
    setResult(res)
  }

  const handleExport = async () => {
    const res = await exportToLLM(filePath, 'compact')
    setResult(res)
  }

  return (
    <div className="semantic-demo">
      <h2>🤖 LLM-Friendly Code Understanding</h2>
      
      <div className="input-group">
        <label>File Path:</label>
        <input
          type="text"
          value={filePath}
          onChange={(e) => setFilePath(e.target.value)}
          placeholder="/path/to/file.rs"
        />
      </div>

      <div className="input-group">
        <label>Natural Language Query:</label>
        <input
          type="text"
          value={queryStr}
          onChange={(e) => setQueryStr(e.target.value)}
          placeholder="function called main"
        />
      </div>

      <div className="button-group">
        <button onClick={handleParse} disabled={loading}>
          {loading ? 'Parsing...' : '📄 Parse File'}
        </button>
        <button onClick={handleQuery} disabled={loading}>
          {loading ? 'Querying...' : '🔍 Natural Query'}
        </button>
        <button onClick={handleExport} disabled={loading}>
          {loading ? 'Exporting...' : '📤 Export to LLM'}
        </button>
      </div>

      {error && (
        <div className="error">{error}</div>
      )}

      {result && (
        <div className="result">
          <pre>{JSON.stringify(result, null, 2)}</pre>
        </div>
      )}

      <div className="info">
        <h3>💡 Example Queries:</h3>
        <ul>
          <li>"function called main" - Find the main function</li>
          <li>"import from std" - Find standard library imports</li>
          <li>"struct named User" - Find User struct definition</li>
          <li>"all tests" - Find all test functions</li>
          <li>"entry point" - Find program entry point</li>
        </ul>      
      </div>
    </div>
  )
}
