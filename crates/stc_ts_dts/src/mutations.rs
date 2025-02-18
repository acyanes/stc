use std::mem::take;

use rnode::{VisitMut, VisitMutWith};
use stc_ts_ast_rnode::{
    RArrayPat, RBindingIdent, RClass, RClassMember, RClassProp, REmptyStmt, RExportDefaultExpr, RFunction, RModule, RModuleItem,
    RObjectPat, RRestPat, RVarDeclarator,
};
use stc_ts_dts_mutations::{
    ClassMemberMut, ClassMut, ClassPropMut, ExportDefaultMut, FunctionMut, ModuleItemMut, Mutations, PatMut, VarDeclMut,
};
use stc_ts_utils::{HasNodeId, MapWithMut};
use swc_common::DUMMY_SP;

pub fn apply_mutations(mutations: &mut Mutations, m: &mut RModule) {
    let mut v = Operator { mutations };
    m.visit_mut_with(&mut v);
}

struct Operator<'a> {
    mutations: &'a mut Mutations,
}

impl VisitMut<Vec<RModuleItem>> for Operator<'_> {
    fn visit_mut(&mut self, items: &mut Vec<RModuleItem>) {
        let mut new = Vec::with_capacity(items.len() * 11 / 10);

        for mut item in items.take() {
            let node_id = item.node_id();

            if let Some(node_id) = node_id {
                if let Some(ModuleItemMut { prepend_stmts, .. }) = self.mutations.for_module_items.get_mut(&node_id) {
                    new.extend(take(prepend_stmts).into_iter().map(RModuleItem::Stmt));
                }
            }

            item.visit_mut_children_with(self);

            new.push(item);

            if let Some(node_id) = node_id {
                if let Some(ModuleItemMut { append_stmts, .. }) = self.mutations.for_module_items.get_mut(&node_id) {
                    new.extend(take(append_stmts).into_iter().map(RModuleItem::Stmt));
                }
            }
        }

        *items = new;
    }
}

impl VisitMut<RVarDeclarator> for Operator<'_> {
    fn visit_mut(&mut self, d: &mut RVarDeclarator) {
        d.visit_mut_children_with(self);

        if let Some(VarDeclMut { remove_init }) = self.mutations.for_var_decls.remove(&d.node_id) {
            if remove_init {
                d.init = None;
            }
        }
    }
}

impl VisitMut<RClass> for Operator<'_> {
    fn visit_mut(&mut self, c: &mut RClass) {
        c.visit_mut_children_with(self);

        if let Some(ClassMut {
            super_class: Some(super_class),
            additional_members: _,
        }) = self.mutations.for_classes.remove(&c.node_id)
        {
            c.super_class = Some(super_class);
        }
    }
}

impl VisitMut<RFunction> for Operator<'_> {
    fn visit_mut(&mut self, f: &mut RFunction) {
        f.visit_mut_children_with(self);

        if let Some(FunctionMut { ret_ty: Some(ret_ty) }) = self.mutations.for_fns.remove(&f.node_id) {
            f.return_type = Some(box ret_ty.into())
        }
    }
}

impl VisitMut<RClassMember> for Operator<'_> {
    fn visit_mut(&mut self, member: &mut RClassMember) {
        let node_id = match member {
            RClassMember::Constructor(c) => c.node_id,
            RClassMember::Method(m) => m.node_id,
            RClassMember::PrivateMethod(m) => m.node_id,
            RClassMember::ClassProp(p) => p.node_id,
            RClassMember::PrivateProp(p) => p.node_id,
            RClassMember::TsIndexSignature(s) => s.node_id,
            RClassMember::StaticBlock(s) => s.node_id,
            RClassMember::AutoAccessor(a) => a.node_id,
            RClassMember::Empty(_) => return,
        };

        if let Some(ClassMemberMut { remove }) = self.mutations.for_class_members.remove(&node_id) {
            if remove {
                *member = RClassMember::Empty(REmptyStmt { span: DUMMY_SP });
                return;
            }
        }

        member.visit_mut_children_with(self);
    }
}

impl VisitMut<RClassProp> for Operator<'_> {
    fn visit_mut(&mut self, p: &mut RClassProp) {
        p.visit_mut_children_with(self);

        if let Some(ClassPropMut { ty: Some(ty) }) = self.mutations.for_class_props.remove(&p.node_id) {
            p.type_ann = Some(box ty.into())
        }
    }
}

impl VisitMut<RBindingIdent> for Operator<'_> {
    fn visit_mut(&mut self, i: &mut RBindingIdent) {
        i.visit_mut_children_with(self);

        if let Some(PatMut { ty, optional }) = self.mutations.for_pats.remove(&i.node_id) {
            if let Some(ty) = ty {
                i.type_ann = Some(box ty.into())
            }
            if let Some(optional) = optional {
                i.id.optional = optional;
            }
        }
    }
}

impl VisitMut<RObjectPat> for Operator<'_> {
    fn visit_mut(&mut self, obj: &mut RObjectPat) {
        obj.visit_mut_children_with(self);

        if let Some(PatMut { ty, optional }) = self.mutations.for_pats.remove(&obj.node_id) {
            if let Some(ty) = ty {
                obj.type_ann = Some(box ty.into())
            }
            if let Some(optional) = optional {
                obj.optional = optional;
            }
        }
    }
}

impl VisitMut<RArrayPat> for Operator<'_> {
    fn visit_mut(&mut self, arr: &mut RArrayPat) {
        arr.visit_mut_children_with(self);

        if let Some(PatMut { ty, optional }) = self.mutations.for_pats.remove(&arr.node_id) {
            if let Some(ty) = ty {
                arr.type_ann = Some(box ty.into())
            }
            if let Some(optional) = optional {
                arr.optional = optional;
            }
        }
    }
}

impl VisitMut<RRestPat> for Operator<'_> {
    fn visit_mut(&mut self, r: &mut RRestPat) {
        r.visit_mut_children_with(self);

        if let Some(PatMut { ty: Some(ty), optional: _ }) = self.mutations.for_pats.remove(&r.node_id) {
            r.type_ann = Some(box ty.into())
        }
    }
}

impl VisitMut<RExportDefaultExpr> for Operator<'_> {
    fn visit_mut(&mut self, export: &mut RExportDefaultExpr) {
        export.visit_mut_children_with(self);

        if let Some(ExportDefaultMut { replace_with: Some(expr) }) = self.mutations.for_export_defaults.remove(&export.node_id) {
            export.expr = expr;
        }
    }
}
