use cranelift::codegen::{
    cursor::{Cursor, FuncCursor},
    ir::{self, entities::*, InstBuilder},
};

use hashlink::LinkedHashMap;

pub fn get_all_uses(func: &mut ir::Function) -> LinkedHashMap<StackSlot, Vec<Inst>> {
    let mut cursor = FuncCursor::new(func);
    let mut map = LinkedHashMap::new();
    let mut inst_to_ss = LinkedHashMap::new();
    let mut val_to_ss = LinkedHashMap::new();
    for (slot, _) in cursor.func.stack_slots.iter() {
        map.insert(slot, vec![]);
    }

    while let Some(_block) = cursor.next_block() {
        while let Some(inst) = cursor.next_inst() {
            use self::ir::InstructionData::*;
            match &cursor.func.dfg[inst] {
                StackLoad {
                    opcode: _,
                    offset: off,
                    stack_slot,
                } => {
                    if *off == ir::immediates::Offset32::new(0) {
                        map.get_mut(stack_slot).unwrap().push(inst);
                        inst_to_ss.insert(inst, *stack_slot);
                        val_to_ss.insert(cursor.func.dfg.inst_results(inst)[0], *stack_slot);
                    } else {
                        // this does not used as value.
                        map.remove(stack_slot);
                    }
                }
                Load {
                    opcode: _,
                    arg,
                    flags: _,
                    offset,
                } => {
                    if let Some(ss) = val_to_ss.get(arg) {
                        if *offset == ir::immediates::Offset32::new(0) {
                            map.get_mut(ss).unwrap().push(inst);
                        } else {
                            map.remove(ss);
                        }
                    } else {
                    }
                    // inst_to_ss.insert(inst, *stack_slot);
                }
                Store {
                    opcode: _,
                    args,
                    flags: _,
                    offset,
                } => {
                    if let Some(ss) = val_to_ss.get(&args[1]) {
                        if *offset == ir::immediates::Offset32::new(0) {
                            map.get_mut(ss).unwrap().push(inst);
                        } else {
                            map.remove(ss);
                        }
                    } else {
                    }
                    // inst_to_ss.insert(inst, *stack_slot);
                }
                StackStore {
                    opcode: _,
                    offset: off,
                    stack_slot,
                    arg: _,
                } => {
                    if *off == ir::immediates::Offset32::new(0) {
                        map.get_mut(stack_slot).unwrap().push(inst);
                        inst_to_ss.insert(inst, *stack_slot);
                    } else {
                        // this does not used as value.
                        map.remove(stack_slot);
                    }
                }
                _ => (),
            }
        }
    }

    map
}

pub struct Mem2Reg<'a> {
    func: &'a mut ir::Function,
}
impl<'a> Mem2Reg<'a> {
    pub fn run(&mut self) {
        let uses = get_all_uses(self.func);
        let mut cursor = FuncCursor::new(self.func);
        use self::ir::InstructionData::*;
        for (_, insns) in uses.iter() {
            let mut s = None;
            for ins in insns.iter() {
                match cursor.func.dfg[*ins] {
                    Load { .. } => {
                        cursor.func.dfg.replace(*ins).copy(s.unwrap());
                    }
                    Store { args, .. } => {
                        s = Some(cursor.func.dfg.replace(*ins).copy(args[0]));
                    }
                    _ => (),
                }
            }
        }
        while let Some(_bb) = cursor.next_block() {
            while let Some(ins) = cursor.next_inst() {
                for (ss, _) in uses.iter() {
                    match cursor.func.dfg[ins] {
                        StackLoad { stack_slot, .. } => {
                            if stack_slot == *ss {
                                cursor.remove_inst();
                            }
                        }
                        StackStore { stack_slot, .. } => {
                            if stack_slot == *ss {
                                cursor.remove_inst();
                            }
                        }
                        _ => (),
                    }
                }
            }
        }
    }
}
use cranelift::codegen::Context;

pub fn optimize(ctx: &mut Context) {
    Mem2Reg {
        func: &mut ctx.func,
    }
    .run();
}
