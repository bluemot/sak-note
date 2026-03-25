import { useEffect, useState, useRef, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import './HexViewer.css'

interface HexViewerProps {
  filePath: string
  fileSize: number
}

interface HexRow {
  offset: number
  hex: string
  ascii: string
}

const BYTES_PER_ROW = 16
const VISIBLE_ROWS = 40

function HexViewer({ filePath, fileSize }: HexViewerProps) {
  const [rows, setRows] = useState<HexRow[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const [currentOffset, setCurrentOffset] = useState(0)
  const [selectedByte, setSelectedByte] = useState<number | null>(null)
  const containerRef = useRef<HTMLDivElement>(null)

  const loadHexData = useCallback(async (offset: number) => {
    try {
      setIsLoading(true)
      const length = Math.min(BYTES_PER_ROW * VISIBLE_ROWS, fileSize - offset)
      const response = await invoke<{ rows: HexRow[] }>('get_hex_view', {
        req: {
          path: filePath,
          start: offset,
          length
        }
      })
      setRows(response.rows)
      setCurrentOffset(offset)
    } catch (err) {
      console.error('Failed to load hex data:', err)
    } finally {
      setIsLoading(false)
    }
  }, [filePath, fileSize])

  useEffect(() => {
    if (filePath) {
      loadHexData(0)
    }
  }, [filePath, loadHexData])

  const handleScroll = useCallback((e: React.UIEvent<HTMLDivElement>) => {
    const { scrollTop, scrollHeight, clientHeight } = e.currentTarget
    const scrollPercent = scrollTop / (scrollHeight - clientHeight)
    const newOffset = Math.floor(scrollPercent * (fileSize - BYTES_PER_ROW * VISIBLE_ROWS))
    
    if (Math.abs(newOffset - currentOffset) > BYTES_PER_ROW * 10) {
      loadHexData(Math.max(0, newOffset))
    }
  }, [currentOffset, fileSize, loadHexData])

  const handleByteClick = (offset: number) => {
    setSelectedByte(offset)
  }

  const goToOffset = (offset: number) => {
    const clampedOffset = Math.max(0, Math.min(offset, fileSize - 1))
    loadHexData(clampedOffset - (clampedOffset % BYTES_PER_ROW))
  }

  if (isLoading && rows.length === 0) {
    return (
      <div className="hex-viewer-loading">
        Loading hex view...
      </div>
    )
  }

  return (
    <div className="hex-viewer">
      <div className="hex-toolbar">
        <span className="hex-title">Hex View</span>
        <div className="hex-nav">
          <button 
            onClick={() => goToOffset(currentOffset - BYTES_PER_ROW * VISIBLE_ROWS)}
            disabled={currentOffset === 0}
          >
            ← Prev
          </button>
          <span className="hex-offset">
            Offset: 0x{currentOffset.toString(16).toUpperCase().padStart(8, '0')}
          </span>
          <button 
            onClick={() => goToOffset(currentOffset + BYTES_PER_ROW * VISIBLE_ROWS)}
            disabled={currentOffset + BYTES_PER_ROW * VISIBLE_ROWS >= fileSize}
          >
            Next →
          </button>
        </div>
        {selectedByte !== null && (
          <span className="hex-selected">
            Selected: 0x{selectedByte.toString(16).toUpperCase().padStart(8, '0')} ({selectedByte})
          </span>
        )}
      </div>

      <div className="hex-header">
        <span className="hex-offset-header">Offset</span>
        <span className="hex-bytes-header">
          {Array.from({ length: BYTES_PER_ROW }, (_, i) => (
            <span key={i} className="hex-byte-header">{i.toString(16).toUpperCase().padStart(2, '0')}</span>
          ))}
        </span>
        <span className="hex-ascii-header">ASCII</span>
      </div>

      <div 
        className="hex-content" 
        ref={containerRef}
        onScroll={handleScroll}
      >
        {rows.map((row) => (
          <div key={row.offset} className="hex-row">
            <span className="hex-offset-cell">
              {row.offset.toString(16).toUpperCase().padStart(8, '0')}
            </span>
            <span className="hex-bytes">
              {row.hex.split(' ').map((byte, idx) => {
                const byteOffset = row.offset + idx
                return (
                  <span
                    key={idx}
                    className={`hex-byte ${selectedByte === byteOffset ? 'selected' : ''}`}
                    onClick={() => handleByteClick(byteOffset)}
                  >
                    {byte}
                  </span>
                )
              })}
            </span>
            <span className="hex-ascii">
              {row.ascii.split('').map((char, idx) => {
                const byteOffset = row.offset + idx
                return (
                  <span
                    key={idx}
                    className={`hex-ascii-char ${selectedByte === byteOffset ? 'selected' : ''}`}
                    onClick={() => handleByteClick(byteOffset)}
                  >
                    {char}
                  </span>
                )
              })}
            </span>
          </div>
        ))}
      </div>
    </div>
  )
}

export default HexViewer
