#[cfg(test)]
mod crud_tests {
    use crate::core::db::{DB, DbDelete, DbGet, DbInsert, DbList, DbUpdate};
    use crate::core::types::document::{
        CreatedTimestamp, Document, DocumentId, DocumentPath, ModifiedTimestamp,
    };
    use crate::core::types::heading::{DocumentHeading, NewDocumentHeading};
    use crate::core::types::link::{
        DocumentLink, DocumentLinkSource, DocumentLinkTarget, NewDocumentLink,
    };
    use crate::core::types::task::{DocumentTask, NewDocumentTask};
    use jiff::Timestamp;
    use std::path::PathBuf;

    fn setup_db() -> DB {
        DB::open(":memory:").expect("Failed to create in-memory database")
    }

    #[test]
    fn test_document_insert_and_list() {
        let mut db = setup_db();

        let doc = Document::new(
            DocumentId("test-doc".to_string()),
            "Test Document".to_string(),
            DocumentPath(PathBuf::from("/test/path.md")),
            12345u32,
            ModifiedTimestamp(Timestamp::now()),
            CreatedTimestamp(Timestamp::now()),
            serde_json::json!({"key": "value"}),
            "Test document body content".to_string(),
        );

        let ids = Document::insert(&mut db, &[doc.clone()]).expect("Failed to insert document");
        assert_eq!(ids.len(), 1);

        let docs = Document::list(&db).expect("Failed to list documents");
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].id.0, "test-doc");
        assert_eq!(docs[0].title, "Test Document");
        assert_eq!(docs[0].path.0, PathBuf::from("/test/path.md"));
        assert_eq!(docs[0].hash, 12345u32);
    }

    #[test]
    fn test_document_get() {
        let mut db = setup_db();

        let doc = Document::new(
            DocumentId("get-test".to_string()),
            "Get Test".to_string(),
            DocumentPath(PathBuf::from("/get/test.md")),
            99999u32,
            ModifiedTimestamp(Timestamp::now()),
            CreatedTimestamp(Timestamp::now()),
            serde_json::json!({"test": true}),
            "Get test body".to_string(),
        );

        Document::insert(&mut db, &[doc]).expect("Failed to insert document");

        let retrieved = Document::get(&mut db, &DocumentId("get-test".to_string()))
            .expect("Failed to get document");
        assert_eq!(retrieved.id.0, "get-test");
        assert_eq!(retrieved.title, "Get Test");
        assert_eq!(retrieved.hash, 99999u32);
    }

    #[test]
    fn test_document_update() {
        let mut db = setup_db();

        let doc1 = Document::new(
            DocumentId("update-test".to_string()),
            "Original Title".to_string(),
            DocumentPath(PathBuf::from("/original.md")),
            111u32,
            ModifiedTimestamp(Timestamp::now()),
            CreatedTimestamp(Timestamp::now()),
            serde_json::json!({"version": 1}),
            "Original body".to_string(),
        );

        Document::insert(&mut db, &[doc1]).expect("Failed to insert document");

        let doc2 = Document::new(
            DocumentId("update-test".to_string()),
            "Updated Title".to_string(),
            DocumentPath(PathBuf::from("/updated.md")),
            222u32,
            ModifiedTimestamp(Timestamp::now()),
            CreatedTimestamp(Timestamp::now()),
            serde_json::json!({"version": 2}),
            "Updated body".to_string(),
        );

        Document::update(&mut db, &[doc2]).expect("Failed to update document");

        let updated = Document::get(&mut db, &DocumentId("update-test".to_string()))
            .expect("Failed to get updated document");
        assert_eq!(updated.title, "Updated Title");
        assert_eq!(updated.hash, 222u32);
        assert_eq!(updated.data["version"], 2);

        let all_docs = Document::list(&db).expect("Failed to list documents");
        assert_eq!(all_docs.len(), 1, "Update should not create duplicates");
    }

    #[test]
    fn test_document_delete() {
        let mut db = setup_db();

        let doc1 = Document::new(
            DocumentId("delete-test-1".to_string()),
            "Delete Test 1".to_string(),
            DocumentPath(PathBuf::from("/delete1.md")),
            111u32,
            ModifiedTimestamp(Timestamp::now()),
            CreatedTimestamp(Timestamp::now()),
            serde_json::json!({}),
            String::new(),
        );

        let doc2 = Document::new(
            DocumentId("delete-test-2".to_string()),
            "Delete Test 2".to_string(),
            DocumentPath(PathBuf::from("/delete2.md")),
            222u32,
            ModifiedTimestamp(Timestamp::now()),
            CreatedTimestamp(Timestamp::now()),
            serde_json::json!({}),
            String::new(),
        );

        Document::insert(&mut db, &[doc1, doc2]).expect("Failed to insert documents");

        Document::delete(&mut db, &[DocumentId("delete-test-1".to_string())])
            .expect("Failed to delete document");

        let remaining = Document::list(&db).expect("Failed to list documents");
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id.0, "delete-test-2");
    }

    #[test]
    fn test_heading_insert_and_list() {
        let mut db = setup_db();

        // First insert a document to reference
        let doc = Document::new(
            DocumentId("doc-with-heading".to_string()),
            "Document With Heading".to_string(),
            DocumentPath(PathBuf::from("/heading.md")),
            111u32,
            ModifiedTimestamp(Timestamp::now()),
            CreatedTimestamp(Timestamp::now()),
            serde_json::json!({}),
            String::new(),
        );
        Document::insert(&mut db, &[doc]).expect("Failed to insert document");

        let heading = NewDocumentHeading {
            document_id: DocumentId("doc-with-heading".to_string()),
            content: "Test Heading".to_string(),
            level: 2,
            metadata: serde_json::json!({"style": "bold"}),
            range_start: 0,
            range_end: 13,
        };

        let ids = DocumentHeading::insert(&mut db, &[heading]).expect("Failed to insert heading");
        assert_eq!(ids.len(), 1);
        assert!(ids[0] > 0, "Should return a valid ID");

        let headings = DocumentHeading::list(&db).expect("Failed to list headings");
        assert_eq!(headings.len(), 1);
        assert_eq!(headings[0].content, "Test Heading");
        assert_eq!(headings[0].document_id.0, "doc-with-heading");
        assert_eq!(headings[0].metadata["style"], "bold");
        assert_eq!(headings[0].range_start, 0);
        assert_eq!(headings[0].range_end, 13);
    }

    #[test]
    fn test_link_insert_and_list() {
        let mut db = setup_db();

        // Insert documents to link between
        let doc1 = Document::new(
            DocumentId("source-doc".to_string()),
            "Source".to_string(),
            DocumentPath(PathBuf::from("/source.md")),
            111u32,
            ModifiedTimestamp(Timestamp::now()),
            CreatedTimestamp(Timestamp::now()),
            serde_json::json!({}),
            String::new(),
        );
        let doc2 = Document::new(
            DocumentId("target-doc".to_string()),
            "Target".to_string(),
            DocumentPath(PathBuf::from("/target.md")),
            222u32,
            ModifiedTimestamp(Timestamp::now()),
            CreatedTimestamp(Timestamp::now()),
            serde_json::json!({}),
            String::new(),
        );
        Document::insert(&mut db, &[doc1, doc2]).expect("Failed to insert documents");

        let link = NewDocumentLink {
            from: DocumentLinkSource::from(DocumentId("source-doc".to_string())),
            to: Some(DocumentLinkTarget::from(DocumentId(
                "target-doc".to_string(),
            ))),
            range_start: 10,
            range_end: 25,
        };

        let ids = DocumentLink::insert(&mut db, &[link]).expect("Failed to insert link");
        assert_eq!(ids.len(), 1);
        assert!(ids[0] > 0, "Should return a valid ID");

        let link_ids = DocumentLink::list(&db).expect("Failed to list links");
        assert_eq!(link_ids.len(), 1);
    }

    #[test]
    fn test_link_insert_with_null_target() {
        let mut db = setup_db();

        // Insert source document
        let doc = Document::new(
            DocumentId("broken-link-doc".to_string()),
            "Broken Link".to_string(),
            DocumentPath(PathBuf::from("/broken.md")),
            111u32,
            ModifiedTimestamp(Timestamp::now()),
            CreatedTimestamp(Timestamp::now()),
            serde_json::json!({}),
            String::new(),
        );
        Document::insert(&mut db, &[doc]).expect("Failed to insert document");

        let link = NewDocumentLink {
            from: DocumentLinkSource::from(DocumentId("broken-link-doc".to_string())),
            to: None,
            range_start: 5,
            range_end: 15,
        };

        let ids =
            DocumentLink::insert(&mut db, &[link]).expect("Failed to insert link with null target");
        assert_eq!(ids.len(), 1);
    }

    #[test]
    fn test_task_insert() {
        let mut db = setup_db();

        // Insert a document to reference
        let doc = Document::new(
            DocumentId("doc-with-tasks".to_string()),
            "Task Document".to_string(),
            DocumentPath(PathBuf::from("/tasks.md")),
            111u32,
            ModifiedTimestamp(Timestamp::now()),
            CreatedTimestamp(Timestamp::now()),
            serde_json::json!({}),
            String::new(),
        );
        Document::insert(&mut db, &[doc]).expect("Failed to insert document");

        let task1 = NewDocumentTask {
            document_id: DocumentId("doc-with-tasks".to_string()),
            parent_id: None,
            checked: false,
            content: "Unchecked task".to_string(),
            range_start: 0,
            range_end: 14,
        };

        let task2 = NewDocumentTask {
            document_id: DocumentId("doc-with-tasks".to_string()),
            parent_id: None,
            checked: true,
            content: "Checked task".to_string(),
            range_start: 15,
            range_end: 27,
        };

        let ids = DocumentTask::insert(&mut db, &[task1, task2]).expect("Failed to insert tasks");
        assert_eq!(ids.len(), 2);
        assert!(ids[0] > 0 && ids[1] > 0, "Should return valid IDs");
    }

    #[test]
    fn test_timestamp_roundtrip() {
        let mut db = setup_db();

        let now = Timestamp::now();
        let doc = Document::new(
            DocumentId("timestamp-test".to_string()),
            "Timestamp Test".to_string(),
            DocumentPath(PathBuf::from("/time.md")),
            111u32,
            ModifiedTimestamp(now),
            CreatedTimestamp(now),
            serde_json::json!({}),
            String::new(),
        );

        Document::insert(&mut db, &[doc]).expect("Failed to insert document");

        let retrieved = Document::get(&mut db, &DocumentId("timestamp-test".to_string()))
            .expect("Failed to get document");

        // Timestamps should be equal (within reasonable precision for SQL storage)
        assert_eq!(retrieved.modified.0, now);
        assert_eq!(retrieved.created.0, now);
    }

    #[test]
    fn test_json_roundtrip() {
        let mut db = setup_db();

        let complex_json = serde_json::json!({
            "string": "value",
            "number": 42,
            "boolean": true,
            "null": null,
            "array": [1, 2, 3],
            "nested": {
                "deep": {
                    "value": "test"
                }
            }
        });

        let doc = Document::new(
            DocumentId("json-test".to_string()),
            "JSON Test".to_string(),
            DocumentPath(PathBuf::from("/json.md")),
            111u32,
            ModifiedTimestamp(Timestamp::now()),
            CreatedTimestamp(Timestamp::now()),
            complex_json.clone(),
            String::new(),
        );

        Document::insert(&mut db, &[doc]).expect("Failed to insert document");

        let retrieved = Document::get(&mut db, &DocumentId("json-test".to_string()))
            .expect("Failed to get document");

        assert_eq!(retrieved.data, complex_json);
    }

    #[test]
    fn test_path_with_special_characters() {
        let mut db = setup_db();

        let doc = Document::new(
            DocumentId("special-path".to_string()),
            "Special Path".to_string(),
            DocumentPath(PathBuf::from("/path/with spaces/and-dashes_underscores.md")),
            111u32,
            ModifiedTimestamp(Timestamp::now()),
            CreatedTimestamp(Timestamp::now()),
            serde_json::json!({}),
            String::new(),
        );

        Document::insert(&mut db, &[doc]).expect("Failed to insert document");

        let retrieved = Document::get(&mut db, &DocumentId("special-path".to_string()))
            .expect("Failed to get document");

        assert_eq!(
            retrieved.path.0,
            PathBuf::from("/path/with spaces/and-dashes_underscores.md")
        );
    }

    #[test]
    fn test_batch_insert() {
        let mut db = setup_db();

        let docs: Vec<Document> = (0..10)
            .map(|i| {
                Document::new(
                    DocumentId(format!("batch-{}", i)),
                    format!("Batch Document {}", i),
                    DocumentPath(PathBuf::from(format!("/batch/{}.md", i))),
                    i as u32,
                    ModifiedTimestamp(Timestamp::now()),
                    CreatedTimestamp(Timestamp::now()),
                    serde_json::json!({"index": i}),
                    format!("Batch body {}", i),
                )
            })
            .collect();

        let ids = Document::insert(&mut db, &docs).expect("Failed to batch insert");
        assert_eq!(ids.len(), 10);

        let all_docs = Document::list(&db).expect("Failed to list documents");
        assert_eq!(all_docs.len(), 10);
    }
}
