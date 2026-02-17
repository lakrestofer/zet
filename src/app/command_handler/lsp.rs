use color_eyre::eyre::eyre;
use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use tower_lsp_server::jsonrpc::{Error as LspError, Result};
use tower_lsp_server::ls_types::request::{
    GotoDeclarationParams, GotoDeclarationResponse, GotoImplementationParams,
    GotoImplementationResponse, GotoTypeDefinitionParams, GotoTypeDefinitionResponse,
};
use tower_lsp_server::ls_types::*;
use tower_lsp_server::{Client, LanguageServer, LspService, Server};
use zet::core::template_engine::{
    render_template, resolve_group_from_cwd, resolve_template_string,
};
use zet::preamble::*;

pub fn handle_command(root: Option<PathBuf>) -> Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let stdin = tokio::io::stdin();
            let stdout = tokio::io::stdout();

            let (service, socket) = LspService::new(|client| Backend { client });
            Server::new(stdin, stdout, socket).serve(service).await;
        });
    Ok(())
}

#[derive(Debug)]
struct Backend {
    client: Client,
}

impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions::default()),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        Ok(Some(CompletionResponse::Array(vec![
            CompletionItem::new_simple("Hello".to_string(), "Some detail".to_string()),
            CompletionItem::new_simple("Bye".to_string(), "More detail".to_string()),
        ])))
    }

    async fn hover(&self, _: HoverParams) -> Result<Option<Hover>> {
        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String("You're hovering!".to_string())),
            range: None,
        }))
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let _ = params;
        log::warn!("got a `textDocument/didOpen` notification, but it is not implemented");
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let _ = params;
        log::warn!("got a `textDocument/didChange` notification, but it is not implemented");
    }

    async fn will_save(&self, params: WillSaveTextDocumentParams) {
        let _ = params;
        log::warn!("got a `textDocument/willSave` notification, but it is not implemented");
    }

    async fn will_save_wait_until(
        &self,
        params: WillSaveTextDocumentParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        let _ = params;
        log::error!("got a `textDocument/willSaveWaitUntil` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let _ = params;
        log::warn!("got a `textDocument/didSave` notification, but it is not implemented");
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let _ = params;
        log::warn!("got a `textDocument/didClose` notification, but it is not implemented");
    }

    // Notebook Document Synchronization

    async fn notebook_did_open(&self, params: DidOpenNotebookDocumentParams) {
        let _ = params;
        log::warn!("got a `notebookDocument/didOpen` notification, but it is not implemented");
    }

    async fn notebook_did_change(&self, params: DidChangeNotebookDocumentParams) {
        let _ = params;
        log::warn!("got a `notebookDocument/didChange` notification, but it is not implemented");
    }

    async fn notebook_did_save(&self, params: DidSaveNotebookDocumentParams) {
        let _ = params;
        log::warn!("got a `notebookDocument/didSave` notification, but it is not implemented");
    }

    async fn notebook_did_close(&self, params: DidCloseNotebookDocumentParams) {
        let _ = params;
        log::warn!("got a `notebookDocument/didClose` notification, but it is not implemented");
    }

    // Language Features

    async fn goto_declaration(
        &self,
        params: GotoDeclarationParams,
    ) -> Result<Option<GotoDeclarationResponse>> {
        let _ = params;
        log::error!("got a `textDocument/declaration` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let _ = params;
        log::error!("got a `textDocument/definition` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn goto_type_definition(
        &self,
        params: GotoTypeDefinitionParams,
    ) -> Result<Option<GotoTypeDefinitionResponse>> {
        let _ = params;
        log::error!("got a `textDocument/typeDefinition` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn goto_implementation(
        &self,
        params: GotoImplementationParams,
    ) -> Result<Option<GotoImplementationResponse>> {
        let _ = params;
        log::error!("got a `textDocument/implementation` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let _ = params;
        log::error!("got a `textDocument/references` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn prepare_call_hierarchy(
        &self,
        params: CallHierarchyPrepareParams,
    ) -> Result<Option<Vec<CallHierarchyItem>>> {
        let _ = params;
        log::error!("got a `textDocument/prepareCallHierarchy` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn incoming_calls(
        &self,
        params: CallHierarchyIncomingCallsParams,
    ) -> Result<Option<Vec<CallHierarchyIncomingCall>>> {
        let _ = params;
        log::error!("got a `callHierarchy/incomingCalls` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn outgoing_calls(
        &self,
        params: CallHierarchyOutgoingCallsParams,
    ) -> Result<Option<Vec<CallHierarchyOutgoingCall>>> {
        let _ = params;
        log::error!("got a `callHierarchy/outgoingCalls` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn prepare_type_hierarchy(
        &self,
        params: TypeHierarchyPrepareParams,
    ) -> Result<Option<Vec<TypeHierarchyItem>>> {
        let _ = params;
        log::error!("got a `textDocument/prepareTypeHierarchy` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn supertypes(
        &self,
        params: TypeHierarchySupertypesParams,
    ) -> Result<Option<Vec<TypeHierarchyItem>>> {
        let _ = params;
        log::error!("got a `typeHierarchy/supertypes` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn subtypes(
        &self,
        params: TypeHierarchySubtypesParams,
    ) -> Result<Option<Vec<TypeHierarchyItem>>> {
        let _ = params;
        log::error!("got a `typeHierarchy/subtypes` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn document_highlight(
        &self,
        params: DocumentHighlightParams,
    ) -> Result<Option<Vec<DocumentHighlight>>> {
        let _ = params;
        log::error!("got a `textDocument/documentHighlight` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn document_link(&self, params: DocumentLinkParams) -> Result<Option<Vec<DocumentLink>>> {
        let _ = params;
        log::error!("got a `textDocument/documentLink` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn document_link_resolve(&self, params: DocumentLink) -> Result<DocumentLink> {
        let _ = params;
        log::error!("got a `documentLink/resolve` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn code_lens(&self, params: CodeLensParams) -> Result<Option<Vec<CodeLens>>> {
        let _ = params;
        log::error!("got a `textDocument/codeLens` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn code_lens_resolve(&self, params: CodeLens) -> Result<CodeLens> {
        let _ = params;
        log::error!("got a `codeLens/resolve` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn folding_range(&self, params: FoldingRangeParams) -> Result<Option<Vec<FoldingRange>>> {
        let _ = params;
        log::error!("got a `textDocument/foldingRange` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn selection_range(
        &self,
        params: SelectionRangeParams,
    ) -> Result<Option<Vec<SelectionRange>>> {
        let _ = params;
        log::error!("got a `textDocument/selectionRange` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let _ = params;
        log::error!("got a `textDocument/documentSymbol` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let _ = params;
        log::error!("got a `textDocument/semanticTokens/full` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn semantic_tokens_full_delta(
        &self,
        params: SemanticTokensDeltaParams,
    ) -> Result<Option<SemanticTokensFullDeltaResult>> {
        let _ = params;
        log::error!(
            "got a `textDocument/semanticTokens/full/delta` request, but it is not implemented"
        );
        Err(LspError::method_not_found())
    }

    async fn semantic_tokens_range(
        &self,
        params: SemanticTokensRangeParams,
    ) -> Result<Option<SemanticTokensRangeResult>> {
        let _ = params;
        log::error!("got a `textDocument/semanticTokens/range` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn inline_value(&self, params: InlineValueParams) -> Result<Option<Vec<InlineValue>>> {
        let _ = params;
        log::error!("got a `textDocument/inlineValue` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn inlay_hint(&self, params: InlayHintParams) -> Result<Option<Vec<InlayHint>>> {
        let _ = params;
        log::error!("got a `textDocument/inlayHint` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn inlay_hint_resolve(&self, params: InlayHint) -> Result<InlayHint> {
        let _ = params;
        log::error!("got a `inlayHint/resolve` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn moniker(&self, params: MonikerParams) -> Result<Option<Vec<Moniker>>> {
        let _ = params;
        log::error!("got a `textDocument/moniker` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn completion_resolve(&self, params: CompletionItem) -> Result<CompletionItem> {
        let _ = params;
        log::error!("got a `completionItem/resolve` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn diagnostic(
        &self,
        params: DocumentDiagnosticParams,
    ) -> Result<DocumentDiagnosticReportResult> {
        let _ = params;
        log::error!("got a `textDocument/diagnostic` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn workspace_diagnostic(
        &self,
        params: WorkspaceDiagnosticParams,
    ) -> Result<WorkspaceDiagnosticReportResult> {
        let _ = params;
        log::error!("got a `workspace/diagnostic` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn signature_help(&self, params: SignatureHelpParams) -> Result<Option<SignatureHelp>> {
        let _ = params;
        log::error!("got a `textDocument/signatureHelp` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let _ = params;
        log::error!("got a `textDocument/codeAction` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn code_action_resolve(&self, params: CodeAction) -> Result<CodeAction> {
        let _ = params;
        log::error!("got a `codeAction/resolve` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn document_color(&self, params: DocumentColorParams) -> Result<Vec<ColorInformation>> {
        let _ = params;
        log::error!("got a `textDocument/documentColor` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn color_presentation(
        &self,
        params: ColorPresentationParams,
    ) -> Result<Vec<ColorPresentation>> {
        let _ = params;
        log::error!("got a `textDocument/colorPresentation` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let _ = params;
        log::error!("got a `textDocument/formatting` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn range_formatting(
        &self,
        params: DocumentRangeFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        let _ = params;
        log::error!("got a `textDocument/rangeFormatting` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn on_type_formatting(
        &self,
        params: DocumentOnTypeFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        let _ = params;
        log::error!("got a `textDocument/onTypeFormatting` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let _ = params;
        log::error!("got a `textDocument/rename` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<PrepareRenameResponse>> {
        let _ = params;
        log::error!("got a `textDocument/prepareRename` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn linked_editing_range(
        &self,
        params: LinkedEditingRangeParams,
    ) -> Result<Option<LinkedEditingRanges>> {
        let _ = params;
        log::error!("got a `textDocument/linkedEditingRange` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    // Workspace Features

    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<WorkspaceSymbolResponse>> {
        let _ = params;
        log::error!("got a `workspace/symbol` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn symbol_resolve(&self, params: WorkspaceSymbol) -> Result<WorkspaceSymbol> {
        let _ = params;
        log::error!("got a `workspaceSymbol/resolve` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        let _ = params;
        log::warn!(
            "got a `workspace/didChangeConfiguration` notification, but it is not implemented"
        );
    }

    async fn did_change_workspace_folders(&self, params: DidChangeWorkspaceFoldersParams) {
        let _ = params;
        log::warn!(
            "got a `workspace/didChangeWorkspaceFolders` notification, but it is not implemented"
        );
    }

    async fn will_create_files(&self, params: CreateFilesParams) -> Result<Option<WorkspaceEdit>> {
        let _ = params;
        log::error!("got a `workspace/willCreateFiles` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn did_create_files(&self, params: CreateFilesParams) {
        let _ = params;
        log::warn!("got a `workspace/didCreateFiles` notification, but it is not implemented");
    }

    async fn will_rename_files(&self, params: RenameFilesParams) -> Result<Option<WorkspaceEdit>> {
        let _ = params;
        log::error!("got a `workspace/willRenameFiles` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn did_rename_files(&self, params: RenameFilesParams) {
        let _ = params;
        log::warn!("got a `workspace/didRenameFiles` notification, but it is not implemented");
    }

    async fn will_delete_files(&self, params: DeleteFilesParams) -> Result<Option<WorkspaceEdit>> {
        let _ = params;
        log::error!("got a `workspace/willDeleteFiles` request, but it is not implemented");
        Err(LspError::method_not_found())
    }

    async fn did_delete_files(&self, params: DeleteFilesParams) {
        let _ = params;
        log::warn!("got a `workspace/didDeleteFiles` notification, but it is not implemented");
    }

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        let _ = params;
        log::warn!(
            "got a `workspace/didChangeWatchedFiles` notification, but it is not implemented"
        );
    }

    async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<LSPAny>> {
        let _ = params;
        log::error!("got a `workspace/executeCommand` request, but it is not implemented");
        Err(LspError::method_not_found())
    }
}
