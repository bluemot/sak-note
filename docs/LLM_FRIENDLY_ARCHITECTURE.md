# LLM-Friendly Editor Architecture

## 核心理念

傳統編輯器是為人類視覺設計的（GUI、圖標、顏色）。
LLM-friendly 編輯器應該是為「結構化理解」設計的。

## 三大設計原則

### 1. Semantic Blocks (語義塊)
不是原始文字，而是有意義的單元：

```json
{
  "type": "function",
  "name": "calculateTotal",
  "signature": "(items: Item[]) => number",
  "purpose": "Calculate total price with tax",
  "dependencies": ["Item", "TAX_RATE"],
  "blocks": [
    { "type": "param_validation", "content": "..." },
    { "type": "calculation", "content": "..." },
    { "type": "return", "content": "..." }
  ]
}
```

### 2. Context-Aware Addressing (上下文感知定址)
不是行號，而是語義位置：

```
❌ 傳統: "第 42 行"
✅ LLM-friendly: "calculateTotal 函數的參數驗證區塊"
✅ 或者: "User 類別的建構子"
✅ 或者: "import React 的那一行"
```

### 3. Intent-Based Operations (意圖導向操作)

```json
// 不是 "刪除第 10-20 行"
{
  "intent": "extract_function",
  "target": "calculateTotal",
  "selection_logic": "從參數驗證到 return",
  "new_name": "validateAndCalculate"
}
```

## API 設計

### 讀取文件（語義化）

```typescript
// 不是 getLine(42)，而是：
interface Document {
  // 語義結構視圖
  getStructure(): SemanticBlock[];
  
  // 自然語言查詢
  find(query: string): Location[];
  // 例: find("處理登入的函數")
  // 例: find("所有使用了 useState 的地方")
  
  // 獲取上下文
  getContext(location: Location, depth: number): Context;
}
```

### 編輯操作（意圖導向）

```typescript
interface Editor {
  // 不是 "在位置 X 插入文字"
  execute(edit: SemanticEdit): Result;
}

type SemanticEdit =
  | { type: "add_function"; signature: string; implementation: string; after?: string }
  | { type: "refactor_extract_method"; selection: Selection; newName: string }
  | { type: "add_import"; module: string; names: string[] }
  | { type: "update_type"; location: Location; newType: string }
  | { type: "add_error_handling"; function: string; errorTypes: string[] }
```

### 對話接口

```typescript
// LLM 和編輯器的對話
interface LLMConversation {
  // LLM 問：這個函數在做什麼？
  explain(location: Location): Explanation;
  
  // LLM 說：幫我重构這段代碼
  refactor(request: RefactorRequest): Edit[];
  
  // LLM 問：改動會影響哪些地方？
  analyzeImpact(edit: Edit): ImpactAnalysis;
}
```

## 具體應用場景

### 1. 自然語言編輯

```
User: "在 User 類別裡加一個 email 欄位，要有驗證"
→ Editor: 
   1. find("User 類別") 
   2. semantic_edit: { type: "add_field", name: "email", type: "string", validation: "email" }
   3. 自動生成：欄位定義 + getter/setter + 驗證邏輯
```

### 2. 智能重構

```
User: "把這個處理訂單的邏輯抽出來變成獨立函數"
→ Editor:
   1. analyze_selection() → 識別邏輯邊界
   2. find_dependencies() → 找出依賴
   3. semantic_edit: { type: "extract_function", ... }
   4. 更新所有引用
```

### 3. 上下文感知的補全

```
User 在寫登入函數...
Editor: "偵測到你在寫登入邏輯，要幫你加上：
   - 密碼雜湊比對
   - JWT Token 生成
   - 錯誤處理
   - 速率限制？"
```

## 技術實現

### AST + 語義分析

```rust
// 不只是 AST，還有語義圖
struct SemanticGraph {
    blocks: Vec<SemanticBlock>,
    relationships: Vec<Relationship>, // depends_on, calls, extends
    intents: Vec<Intent>, // 這段代碼的意圖是什麼
}
```

### 增量同步

```typescript
// 不是重傳整個文件
interface Delta {
  semantic_change: SemanticEdit,
  affected_blocks: BlockId[],
  verification: VerificationResult
}
```

## SAK Editor 整合規劃

我們現在有的：
- ✅ VFS 文件系統
- ✅ Module 系統（file, llm, sftp）
- ✅ React + Tauri 架構

可以加的：

### Phase 1: Semantic Parser
```rust
// 新增 semantic 模組
pub mod semantic {
    pub struct CodeParser;
    pub struct SemanticBlock;
    pub fn parse_to_blocks(content: &str) -> Vec<SemanticBlock>;
}
```

### Phase 2: LLM-Aware API
```typescript
// 新增 llm-editor bridge
interface LLMEditorBridge {
  // 把文件轉成 LLM 友善格式
  exportForLLM(): LLMFormat;
  
  // 把 LLM 的回應轉成編輯操作
  applyLLMEdit(llmResponse: string): EditResult;
}
```

### Phase 3: Conversational UI
```typescript
// 新增聊天式編輯界面
// 不是傳統編輯器，而是對話框 + 預覽
```

## 市場定位

| 編輯器 | 特點 |
|--------|------|
| VS Code | 人類為主，插件豐富 |
| Cursor | AI 輔助，但仍傳統編輯 |
| **SAK LLM Editor** | **AI 原生，語義導向** |

**殺手鐧**：LLM 可以直接 "理解" 和 "操作" 文件，不需要人類翻譯。

---

要開始實現嗎？我可以先從 Phase 1 (Semantic Parser) 開始！