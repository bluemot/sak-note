/**
 * SearchPanel Component
 *
 * Bottom panel for displaying search results with navigation,
 * highlighting, and collapsible state.
 */

import React, { useState, useCallback, useRef, useEffect } from 'react';
import './SearchPanel.css';

export interface SearchResult {
  id: string;
  line: number;
  column: number;
  text: string;
  context: string;
  filePath?: string;
}

export interface SearchPanelProps {
  query?: string;
  results?: SearchResult[];
  onResultClick?: (result: SearchResult) => void;
  onNavigate?: (direction: 'prev' | 'next') => void;
  onClose?: () => void;
}

export const SearchPanel: React.FC<SearchPanelProps> = ({
  query = '',
  results = [],
  onResultClick,
  onNavigate,
  onClose,
}) => {
  const [isCollapsed, setIsCollapsed] = useState(false);
  const [currentIndex, setCurrentIndex] = useState(0);
  const [filterText, setFilterText] = useState('');
  const resultsListRef = useRef<HTMLDivElement>(null);
  const activeResultRef = useRef<HTMLDivElement>(null);

  // Update current index when results change
  useEffect(() => {
    setCurrentIndex(0);
  }, [results]);

  // Scroll active result into view
  useEffect(() => {
    if (activeResultRef.current && resultsListRef.current) {
      activeResultRef.current.scrollIntoView({
        behavior: 'smooth',
        block: 'nearest',
      });
    }
  }, [currentIndex]);

  const handleToggleCollapse = useCallback(() => {
    setIsCollapsed((prev) => !prev);
  }, []);

  const handlePrev = useCallback(() => {
    if (results.length === 0) return;
    const newIndex = currentIndex > 0 ? currentIndex - 1 : results.length - 1;
    setCurrentIndex(newIndex);
    onNavigate?.('prev');
  }, [currentIndex, results.length, onNavigate]);

  const handleNext = useCallback(() => {
    if (results.length === 0) return;
    const newIndex = currentIndex < results.length - 1 ? currentIndex + 1 : 0;
    setCurrentIndex(newIndex);
    onNavigate?.('next');
  }, [currentIndex, results.length, onNavigate]);

  const handleResultClick = useCallback(
    (result: SearchResult, index: number) => {
      setCurrentIndex(index);
      onResultClick?.(result);
    },
    [onResultClick]
  );

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose?.();
      } else if (e.key === 'F3' || (e.key === 'g' && e.ctrlKey)) {
        e.preventDefault();
        handleNext();
      } else if (e.key === 'F3' && e.shiftKey) {
        e.preventDefault();
        handlePrev();
      }
    },
    [handleNext, handlePrev, onClose]
  );

  // Filter results by text
  const filteredResults = filterText
    ? results.filter(
        (r) =>
          r.text.toLowerCase().includes(filterText.toLowerCase()) ||
          r.context.toLowerCase().includes(filterText.toLowerCase())
      )
    : results;

  // Highlight matched text
  const highlightMatch = (text: string, query: string): React.ReactNode => {
    if (!query) return text;

    const parts = text.split(new RegExp(`(${escapeRegExp(query)})`, 'gi'));
    return parts.map((part, index) => {
      if (part.toLowerCase() === query.toLowerCase()) {
        return (
          <mark key={index} className="search-panel__highlight">
            {part}
          </mark>
        );
      }
      return part;
    });
  };

  const escapeRegExp = (string: string): string => {
    return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  };

  if (isCollapsed) {
    return (
      <div className="search-panel search-panel--collapsed">
        <div className="search-panel__header">
          <button
            className="search-panel__expand-btn"
            onClick={handleToggleCollapse}
            title="Expand search results"
          >
            <span className="search-panel__icon">▲</span>
            <span>Search Results ({results.length})</span>
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="search-panel" onKeyDown={handleKeyDown}>
      <div className="search-panel__header">
        <div className="search-panel__title">
          <button
            className="search-panel__collapse-btn"
            onClick={handleToggleCollapse}
            title="Collapse"
          >
            <span className="search-panel__icon">▼</span>
          </button>
          <span>Search Results</span>
          {query && (
            <span className="search-panel__query">"{query}"</span>
          )}
          <span className="search-panel__count">
            {filteredResults.length} result{filteredResults.length !== 1 ? 's' : ''}
          </span>
        </div>

        <div className="search-panel__controls">
          {filteredResults.length > 0 && (
            <div className="search-panel__navigation">
              <span className="search-panel__current">
                {currentIndex + 1} / {filteredResults.length}
              </span>
              <button
                className="search-panel__nav-btn"
                onClick={handlePrev}
                disabled={filteredResults.length === 0}
                title="Previous result (Shift+F3)"
              >
                ◀ Previous
              </button>
              <button
                className="search-panel__nav-btn"
                onClick={handleNext}
                disabled={filteredResults.length === 0}
                title="Next result (F3)"
              >
                Next ▶
              </button>
            </div>
          )}

          <div className="search-panel__filter">
            <input
              type="text"
              placeholder="Filter results..."
              value={filterText}
              onChange={(e) => setFilterText(e.target.value)}
              className="search-panel__filter-input"
            />
          </div>

          <button
            className="search-panel__close-btn"
            onClick={onClose}
            title="Close (Escape)"
          >
            ✕
          </button>
        </div>
      </div>

      <div className="search-panel__results" ref={resultsListRef}>
        {filteredResults.length === 0 ? (
          <div className="search-panel__empty">
            {query ? 'No results found' : 'Enter search query to see results'}
          </div>
        ) : (
          filteredResults.map((result, index) => (
            <div
              key={result.id}
              ref={index === currentIndex ? activeResultRef : null}
              className={`search-panel__result ${
                index === currentIndex ? 'search-panel__result--active' : ''
              }`}
              onClick={() => handleResultClick(result, index)}
              onDoubleClick={() => {
                handleResultClick(result, index);
                onNavigate?.('next');
              }}
            >
              <div className="search-panel__result-location">
                {result.filePath && (
                  <span className="search-panel__result-file">{result.filePath}</span>
                )}
                <span className="search-panel__result-line">
                  Line {result.line}, Col {result.column}
                </span>
              </div>
              <div className="search-panel__result-text">
                {highlightMatch(result.context, query)}
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
};

// Hook for search panel state
export function useSearchPanel(): {
  isVisible: boolean;
  show: () => void;
  hide: () => void;
  toggle: () => void;
} {
  const [isVisible, setIsVisible] = useState(false);

  return {
    isVisible,
    show: () => setIsVisible(true),
    hide: () => setIsVisible(false),
    toggle: () => setIsVisible((v) => !v),
  };
}

export default SearchPanel;
