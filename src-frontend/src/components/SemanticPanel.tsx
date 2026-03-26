interface SemanticPanelProps {
  filePath: string | null
}

export default function SemanticPanel({ filePath }: SemanticPanelProps) {
  return (
    <div className="semantic-panel">
      <h3>Semantic Analysis</h3>
      {filePath ? (
        <div>
          <p>File: {filePath}</p>
          <p>Semantic analysis placeholder - to be implemented</p>
        </div>
      ) : (
        <p>No file selected</p>
      )}
    </div>
  )
}
