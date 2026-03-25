#[cfg(test)]
mod tests {
    use crate::vfs::{EditJournal, EditOp, Piece};

    fn create_test_journal() -> EditJournal {
        EditJournal::new()
    }

    #[test]
    fn test_new_journal_is_clean() {
        let journal = create_test_journal();
        assert!(!journal.is_dirty());
        assert!(!journal.can_undo());
        assert!(!journal.can_redo());
    }

    #[test]
    fn test_insert_operation() {
        let journal = create_test_journal();
        let op = EditOp::Insert { offset: 0, data: vec![1, 2, 3] };
        
        journal.add_edit(op).unwrap();
        
        assert!(journal.is_dirty());
        assert!(journal.can_undo());
        assert!(!journal.can_redo());
    }

    #[test]
    fn test_delete_operation() {
        let journal = create_test_journal();
        let op = EditOp::Delete { offset: 5, length: 10 };
        
        journal.add_edit(op).unwrap();
        
        assert!(journal.is_dirty());
        assert!(journal.can_undo());
    }

    #[test]
    fn test_write_operation() {
        let journal = create_test_journal();
        let op = EditOp::Write { offset: 0, data: vec![104, 101, 108, 108, 111] }; // "hello"
        
        journal.add_edit(op).unwrap();
        
        assert!(journal.is_dirty());
        assert!(journal.can_undo());
    }

    #[test]
    fn test_undo_redo() {
        let journal = create_test_journal();
        
        // Add an edit
        journal.add_edit(EditOp::Insert { offset: 0, data: vec![1] }).unwrap();
        assert!(journal.can_undo());
        
        // Undo
        assert!(journal.undo());
        assert!(!journal.can_undo());
        assert!(journal.can_redo());
        
        // Redo
        assert!(journal.redo());
        assert!(journal.can_undo());
        assert!(!journal.can_redo());
    }

    #[test]
    fn test_build_piece_table_no_edits() {
        let journal = create_test_journal();
        let pieces = journal.build_piece_table(100);
        
        assert_eq!(pieces.len(), 1);
        match &pieces[0] {
            Piece::Original { offset, length } => {
                assert_eq!(*offset, 0);
                assert_eq!(*length, 100);
            }
            _ => panic!("Expected Original piece"),
        }
    }

    #[test]
    fn test_piece_table_with_insert() {
        let journal = create_test_journal();
        journal.add_edit(EditOp::Insert { offset: 5, data: vec![1, 2, 3] }).unwrap();
        
        let pieces = journal.build_piece_table(10);
        
        assert!(pieces.len() >= 2);
        // Should have: [Original(0-5), Added(1,2,3), Original(5-10)]
    }

    #[test]
    fn test_piece_table_with_delete() {
        let journal = create_test_journal();
        journal.add_edit(EditOp::Delete { offset: 3, length: 4 }).unwrap();
        
        let pieces = journal.build_piece_table(10);
        
        // Deleted 4 bytes from offset 3, so effective size should be 6
        let effective: u64 = pieces.iter().map(|p| p.length()).sum();
        assert_eq!(effective, 6);
    }

    #[test]
    fn test_effective_size() {
        let journal = create_test_journal();
        
        // Original size 100
        // Insert 10 at offset 50 -> size 110
        journal.add_edit(EditOp::Insert { offset: 50, data: vec![0; 10] }).unwrap();
        assert_eq!(journal.effective_size(100), 110);
        
        // Delete 20 at offset 10 -> size 90
        journal.add_edit(EditOp::Delete { offset: 10, length: 20 }).unwrap();
        assert_eq!(journal.effective_size(100), 90);
    }

    #[test]
    fn test_logical_to_physical_mapping() {
        let journal = create_test_journal();
        
        // No edits: logical = physical
        assert_eq!(journal.logical_to_physical(0, 100), Some(0));
        assert_eq!(journal.logical_to_physical(50, 100), Some(50));
        assert_eq!(journal.logical_to_physical(99, 100), Some(99));
    }

    #[test]
    fn test_clear_journal() {
        let journal = create_test_journal();
        
        journal.add_edit(EditOp::Insert { offset: 0, data: vec![1] }).unwrap();
        journal.add_edit(EditOp::Insert { offset: 1, data: vec![2] }).unwrap();
        
        assert!(journal.has_edits());
        
        journal.clear();
        
        assert!(!journal.has_edits());
        assert!(!journal.can_undo());
        assert!(!journal.can_redo());
    }

    #[test]
    fn test_clone_journal() {
        let journal = create_test_journal();
        journal.add_edit(EditOp::Insert { offset: 0, data: vec![1, 2, 3] }).unwrap();
        
        let cloned = journal.clone();
        
        assert_eq!(cloned.can_undo(), journal.can_undo());
        assert_eq!(cloned.is_dirty(), journal.is_dirty());
    }
}
