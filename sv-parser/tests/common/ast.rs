use sv_parser::{ExprRef, ModuleItem, ModuleItemRef, ProceduralBlockType, SourceUnit, StmtRef};

/// Return the module item references contained within the module at `module_index`.
#[allow(dead_code)]
pub fn module_items(unit: &SourceUnit, module_index: usize) -> Vec<ModuleItemRef> {
    match unit.module_item_arena.get(unit.items[module_index]) {
        ModuleItem::ModuleDeclaration { items, .. } => items.clone(),
        other => panic!("Expected module declaration, got {:?}", other),
    }
}

/// Return the expression reference for the assignment stored at the given module/item indices.
#[allow(dead_code)]
pub fn assignment_expr(unit: &SourceUnit, module_index: usize, item_index: usize) -> ExprRef {
    let items = module_items(unit, module_index);
    let item_ref = items
        .get(item_index)
        .unwrap_or_else(|| panic!("Missing module item {}", item_index));
    match unit.module_item_arena.get(*item_ref) {
        ModuleItem::Assignment { expr, .. } => *expr,
        other => panic!("Expected assignment, got {:?}", other),
    }
}

/// Convenience wrapper for the first assignment in the first module.
#[allow(dead_code)]
pub fn first_assignment_expr(unit: &SourceUnit) -> ExprRef {
    assignment_expr(unit, 0, 0)
}

/// Return the statements for the first initial block inside a module.
#[allow(dead_code)]
pub fn initial_block_statements(unit: &SourceUnit, module_index: usize) -> Vec<StmtRef> {
    let items = module_items(unit, module_index);

    items
        .into_iter()
        .find_map(|item_ref| match unit.module_item_arena.get(item_ref) {
            ModuleItem::ProceduralBlock {
                block_type,
                statements,
                ..
            } if *block_type == ProceduralBlockType::Initial => Some(statements.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("Expected initial block in module {}", module_index))
}

/// Convenience wrapper for the first initial block in the first module.
#[allow(dead_code)]
pub fn first_initial_block_statements(unit: &SourceUnit) -> Vec<StmtRef> {
    initial_block_statements(unit, 0)
}
