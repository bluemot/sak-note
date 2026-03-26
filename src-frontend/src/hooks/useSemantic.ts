import { useState, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'

export interface SemanticBlock {
  id: string
  type: string
  name: string
  signature?: string
  purpose?: string
  line_start: number
  line_end: number
  preview?: string
}

export interface ParseResult {
  path: string
  language: string
  summary: string
  block_count: number
  blocks: SemanticBlock[]
  errors: Array<{ line: number; message: string }>
}

export interface QueryResult {
  query: string
  total: number
  results: SemanticBlock[]
}

export interface LLMFormat {
  format: string
  content: string
}

export function useSemantic() {
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const parseFile = useCallback(async (path: string): Promise<ParseResult | null> => {
    setLoading(true)
    setError(null)
    try {
      const result = await invoke<ParseResult>('semantic_parse_file', { path })
      return result
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
      return null
    } finally {
      setLoading(false)
    }
  }, [])

  const query = useCallback(async (path: string, queryStr: string): Promise<QueryResult | null> => {
    setLoading(true)
    setError(null)
    try {
      const result = await invoke<QueryResult>('semantic_query', { path, query: queryStr })
      return result
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
      return null
    } finally {
      setLoading(false)
    }
  }, [])

  const exportToLLM = useCallback(async (path: string, format: 'json' | 'compact' | 'tree' = 'compact'): Promise<LLMFormat | null> => {
    setLoading(true)
    setError(null)
    try {
      const result = await invoke<LLMFormat>('semantic_export_llm', { path, format })
      return result
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
      return null
    } finally {
      setLoading(false)
    }
  }, [])

  const startConversation = useCallback(async (): Promise<string | null> => {
    try {
      const id = await invoke<string>('semantic_conversation_start')
      return id
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
      return null
    }
  }, [])

  const sendMessage = useCallback(async (conversationId: string, message: string, fileContext?: string) => {
    setLoading(true)
    setError(null)
    try {
      const result = await invoke<{ response: string; suggestions: string[] }>('semantic_conversation_send', {
        conversationId,
        message,
        fileContext,
      })
      return result
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
      return null
    } finally {
      setLoading(false)
    }
  }, [])

  return {
    loading,
    error,
    parseFile,
    query,
    exportToLLM,
    startConversation,
    sendMessage,
  }
}
