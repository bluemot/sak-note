import { useState, useCallback } from 'react'
import { MDXEditor, 
  headingsPlugin, 
  listsPlugin, 
  quotePlugin, 
  thematicBreakPlugin,
  markdownShortcutPlugin,
  linkPlugin,
  imagePlugin,
  tablePlugin,
  codeBlockPlugin,
  codeMirrorPlugin,
  diffSourcePlugin,
  toolbarPlugin,
  BlockTypeSelect,
  BoldItalicUnderlineToggles,
  CodeToggle,
  CreateLink,
  InsertCodeBlock,
  InsertImage,
  InsertTable,
  InsertThematicBreak,
  ListsToggle,
  UndoRedo,
  frontmatterPlugin,
  linkDialogPlugin
} from '@mdxeditor/editor'
import '@mdxeditor/editor/style.css'
import './MarkdownEditor.css'

interface MarkdownEditorProps {
  filePath: string
  initialContent: string
  onChange?: (content: string) => void
}

function MarkdownEditor({ filePath, initialContent, onChange }: MarkdownEditorProps) {
  const [content, setContent] = useState(initialContent)
  const [viewMode, setViewMode] = useState<'rich-text' | 'source'>('rich-text')

  const handleChange = useCallback((newContent: string) => {
    setContent(newContent)
    onChange?.(newContent)
  }, [onChange])

  // MDXEditor plugins configuration
  const plugins = [
    // Core editing plugins
    headingsPlugin(),
    listsPlugin(),
    quotePlugin(),
    thematicBreakPlugin(),
    markdownShortcutPlugin(),
    
    // Link and media
    linkPlugin(),
    linkDialogPlugin(),
    imagePlugin(),
    
    // Tables and code
    tablePlugin(),
    codeBlockPlugin({ defaultCodeBlockLanguage: 'txt' }),
    codeMirrorPlugin({
      codeBlockLanguages: {
        js: 'JavaScript',
        ts: 'TypeScript',
        jsx: 'JSX',
        tsx: 'TSX',
        css: 'CSS',
        html: 'HTML',
        python: 'Python',
        rust: 'Rust',
        bash: 'Bash',
        json: 'JSON',
        yaml: 'YAML',
        markdown: 'Markdown',
        txt: 'Plain Text'
      }
    }),
    
    // Frontmatter support
    frontmatterPlugin(),
    
    // Source/diff view
    diffSourcePlugin({
      viewMode: viewMode,
      diffMarkdown: initialContent
    }),
    
    // Toolbar
    toolbarPlugin({
      toolbarContents: () => (
        <>
          <UndoRedo />
          <BlockTypeSelect />
          <BoldItalicUnderlineToggles />
          <CodeToggle />
          <CreateLink />
          <InsertImage />
          <InsertTable />
          <InsertThematicBreak />
          <ListsToggle />
          <InsertCodeBlock />
        </>
      )
    })
  ]

  return (
    <div className="markdown-editor-container">
      <div className="markdown-editor-header">
        <span className="file-path">{filePath}</span>
        <div className="view-mode-toggle">
          <button 
            className={viewMode === 'rich-text' ? 'active' : ''}
            onClick={() => setViewMode('rich-text')}
          >
            Rich Text
          </button>
          <button 
            className={viewMode === 'source' ? 'active' : ''}
            onClick={() => setViewMode('source')}
          >
            Source
          </button>
        </div>
      </div>
      
      <div className="markdown-editor-content">
        <MDXEditor
          markdown={content}
          onChange={handleChange}
          plugins={plugins}
          className="mdx-editor"
          contentEditableClassName="mdx-editor-content"
        />
      </div>
      
      <div className="markdown-editor-footer">
        <span>{content.length} characters</span>
        <span>{content.split(/\s+/).filter(w => w.length > 0).length} words</span>
        <span>{content.split('\n').length} lines</span>
      </div>
    </div>
  )
}

export default MarkdownEditor
