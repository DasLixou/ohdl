use std::collections::{hash_map::Entry, HashMap};

use bumpalo::Bump;

use crate::{
    ast::{self, Item},
    hir::{Declaration, Enum, Record, TypeResolvingScope, Variant, HIR},
    message::Message,
    MESSAGES,
};

pub struct RoughLowering<'a, 'hir> {
    pub arena: &'hir Bump,
    pub hir: &'a mut HIR<'hir>,
}

impl<'hir> RoughLowering<'_, 'hir> {
    pub fn lower(&mut self, items: &[ast::Item]) {
        let root_scope = self.hir.tr_scopes.insert(TypeResolvingScope {
            parent: None,
            types: HashMap::new(),
        });
        for item in items {
            self.lower_item(item, root_scope);
        }
    }

    pub fn lower_item(&mut self, item: &Item, scope: usize) {
        match &item.base.0 {
            ast::ItemBase::Record(r) => self.introduce_type(scope, |type_id| {
                Declaration::Record(Record {
                    type_id,
                    name: r.name,
                })
            }),
            ast::ItemBase::Enum(e) => self.introduce_type(scope, |type_id| {
                Declaration::Enum(Enum {
                    type_id,
                    name: e.name,
                    variants: self
                        .arena
                        .alloc_slice_fill_iter(e.variants.iter().map(|&ident| Variant { ident })),
                })
            }),
            _ => { /* TODO */ }
        }
    }

    fn introduce_type<F>(&mut self, scope: usize, f: F)
    where
        F: FnOnce(usize) -> Declaration<'hir>,
    {
        let id = self.hir.types.insert_with(f);
        let name = self.hir.types[id].name();
        match self.hir.tr_scopes[scope].types.entry(name.0) {
            Entry::Vacant(entry) => {
                entry.insert(id);
            }
            Entry::Occupied(entry) => {
                let original = self.hir.types[*entry.get()].name();
                MESSAGES.report(Message::already_in_scope(name.0.get(), name.1, original.1));
            }
        }
    }
}
