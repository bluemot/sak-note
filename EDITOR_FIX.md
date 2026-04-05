# Editor Virtual Scrolling Fix

## Problem
Current implementation:
```typescript
setContent(prev => prev + text)  // This triggers React re-render, resets scroll
```

When scrolling to bottom:
1. User scrolls near bottom
2. `loadNextChunk()` is called
3. `setContent()` appends new text
4. React re-renders Editor component
5. Monaco Editor resets to top (scroll position lost)

## Solution

Direct Monaco model manipulation:

```typescript
const loadNextChunk = useCallback(async () => {
  if (chunkRange.end >= fileSize || !editorRef.current) return

  const editor = editorRef.current
  const monaco = monacoRef.current
  const newStart = chunkRange.end
  const newEnd = Math.min(newStart + CHUNK_SIZE, fileSize)

  try {
    const text = await invoke<string>('get_text', {
      req: { path: filePath, start: newStart, end: newEnd }
    })

    // Save scroll position
    const scrollTop = editor.getScrollTop()
    
    // Direct model manipulation - no React re-render
    const model = editor.getModel()
    if (model && monaco) {
      const lineCount = model.getLineCount()
      const lastLineLength = model.getLineMaxColumn(lineCount)
      
      // Append text at end
      model.applyEdits([{
        range: new monaco.Range(lineCount, lastLineLength, lineCount, lastLineLength),
        text: text
      }])
      
      // Restore scroll position
      editor.setScrollTop(scrollTop)
    }

    setChunkRange({ start: chunkRange.start, end: newEnd })
  } catch (err) {
    log('Error loading chunk:', err)
  }
}, [filePath, fileSize, chunkRange])
```

## Key Changes

1. **Remove `content` state** - Don't use React state for large file content
2. **Direct model access** - Use `editor.getModel().applyEdits()`
3. **Preserve scroll** - Save/restore `editor.getScrollTop()`
4. **Only track chunkRange** - For knowing what range is loaded

## Alternative: Virtual Document

For true virtual scrolling (only render visible lines):
- Use Monaco's `setLazyProxy` or custom virtual document
- Implement `ITextModel` interface
- Only provide content for visible lines on demand

This is more complex but scales to TB-sized files.
