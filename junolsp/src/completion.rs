use tower_lsp::{ jsonrpc::Result, lsp_types::* };
use libjuno::{ BuiltinEnum, REGISTRY };
use crate::backend::Backend;

pub async fn completion(
    _backend: &Backend,
    _params: CompletionParams
) -> Result<Option<CompletionResponse>> {
    let mut items = keywords();
    items.append(&mut builtins());
    items.append(&mut snippets());

    Ok(Some(CompletionResponse::Array(items)))
}
fn snippets() -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: "fn".into(),
            kind: Some(CompletionItemKind::SNIPPET),
            insert_text: Some("fn ${1:name}(${2}) {\n\t$0\n}".into()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            detail: Some("Function".into()),
            ..Default::default()
        },
        CompletionItem {
            label: "let".into(),
            kind: Some(CompletionItemKind::SNIPPET),
            insert_text: Some("let ${1:mut} ${2:name} = $0;".into()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            detail: Some("Let".into()),
            ..Default::default()
        },
        CompletionItem {
            label: "return".into(),
            kind: Some(CompletionItemKind::SNIPPET),
            insert_text: Some("return $0;".into()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            detail: Some("Return a value".into()),
            ..Default::default()
        },
        CompletionItem {
            label: "break".into(),
            kind: Some(CompletionItemKind::SNIPPET),
            insert_text: Some("break;".into()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            detail: Some("Break a loop".into()),
            ..Default::default()
        },
        CompletionItem {
            label: "continue".into(),
            kind: Some(CompletionItemKind::SNIPPET),
            insert_text: Some("continue;".into()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            detail: Some("Continue in a loop".into()),
            ..Default::default()
        },
        CompletionItem {
            label: "loop".into(),
            kind: Some(CompletionItemKind::SNIPPET),
            insert_text: Some("loop {\n\t$0\n}".into()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            detail: Some("Loop".into()),
            ..Default::default()
        },
        CompletionItem {
            label: "while".into(),
            kind: Some(CompletionItemKind::SNIPPET),
            insert_text: Some("while ($1) {\n\t$0\n}".into()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            detail: Some("While".into()),
            ..Default::default()
        },
        CompletionItem {
            label: "if".into(),
            kind: Some(CompletionItemKind::SNIPPET),
            insert_text: Some("if ($1) {\n\t$0\n}".into()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            detail: Some("If".into()),
            ..Default::default()
        },
        CompletionItem {
            label: "elif".into(),
            kind: Some(CompletionItemKind::SNIPPET),
            insert_text: Some("else if ($1) {\n\t$0\n}".into()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            detail: Some("else if".into()),
            ..Default::default()
        },
        CompletionItem {
            label: "else".into(),
            kind: Some(CompletionItemKind::SNIPPET),
            insert_text: Some("else {\n\t$0\n}".into()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            detail: Some("else".into()),
            ..Default::default()
        }
    ]
}
fn builtins() -> Vec<CompletionItem> {
    let mut items = vec![];

    for (name, i) in REGISTRY.entries().into_iter() {
        items.push(match i.declare {
            BuiltinEnum::Function { param_types, return_type: _ } => {
                let param_len = param_types.unwrap_or(&[]).len();
                let mut param_text_v = vec![];
                for p in 0..param_len - 1 {
                    param_text_v.push(format!("${}", p));
                }
                let param_text = param_text_v.join(", ");
                let insert_text = format!("{name}({param_text});");
                CompletionItem {
                    label: (*name).into(),
                    kind: Some(CompletionItemKind::FUNCTION),
                    detail: Some("Builtin".into()),
                    insert_text: Some(insert_text),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    ..Default::default()
                }
            }
            _ => {
                continue;
            }
        });
    }
    items
}

fn keywords() -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: "fn".into(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Function".into()),
            ..Default::default()
        },
        CompletionItem {
            label: "loop".into(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Loop".into()),
            ..Default::default()
        },

        CompletionItem {
            label: "let".into(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Variable".into()),
            ..Default::default()
        },
        CompletionItem {
            label: "mut".into(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Mutable".into()),
            ..Default::default()
        },
        CompletionItem {
            label: "if".into(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("If".into()),
            ..Default::default()
        },
        CompletionItem {
            label: "else".into(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Else".into()),
            ..Default::default()
        },
        CompletionItem {
            label: "while".into(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("While".into()),
            ..Default::default()
        },
        CompletionItem {
            label: "for".into(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("For".into()),
            ..Default::default()
        },
        CompletionItem {
            label: "return".into(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Return".into()),
            ..Default::default()
        },
        CompletionItem {
            label: "break".into(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Break".into()),
            ..Default::default()
        },
        CompletionItem {
            label: "continue".into(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Continue".into()),
            ..Default::default()
        },
        CompletionItem {
            label: "import".into(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Import".into()),
            ..Default::default()
        },

        CompletionItem {
            label: "struct".into(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Struct".into()),
            ..Default::default()
        }
    ]
}
