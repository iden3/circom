use super::ir_interface::*;
use crate::translating_traits::*;
use code_producers::c_elements::*;
use code_producers::llvm_elements::{LLVMInstruction, LLVMIRProducer};
use code_producers::llvm_elements::functions::create_bb;
use code_producers::llvm_elements::instructions::{create_br, create_conditional_branch};
use code_producers::wasm_elements::*;
use program_structure::ast::Statement;

#[derive(Clone, Debug)]
pub struct LoopBucket {
    pub stmt: Statement,
    pub line: usize,
    pub message_id: usize,
    pub continue_condition: InstructionPointer,
    pub body: InstructionList,
}

impl IntoInstruction for LoopBucket {
    fn into_instruction(self) -> Instruction {
        Instruction::Loop(self)
    }
}

impl Allocate for LoopBucket {
    fn allocate(self) -> InstructionPointer {
        InstructionPointer::new(self.into_instruction())
    }
}

impl ObtainMeta for LoopBucket {
    fn get_line(&self) -> usize {
        self.line
    }
    fn get_message_id(&self) -> usize {
        self.message_id
    }
}

impl ToString for LoopBucket {
    fn to_string(&self) -> String {
        let line = self.line.to_string();
        let template_id = self.message_id.to_string();
        let cond = self.continue_condition.to_string();
        let mut body = "".to_string();
        for i in &self.body {
            body = format!("{}{};", body, i.to_string());
        }
        format!("LOOP(line:{},template_id:{},cond:{},body:{})", line, template_id, cond, body)
    }
}

impl WriteLLVMIR for LoopBucket {
    fn produce_llvm_ir<'a, 'b>(&self, producer: &'b dyn LLVMIRProducer<'a>) -> Option<LLVMInstruction<'a>> {
        let current_function = producer.current_function();
        let cond_bb = create_bb(producer, current_function, "loop.cond.0");
        let body_bb = create_bb(producer, current_function, "loop.body.0");
        let end_bb = create_bb(producer, current_function, "loop.end.0");
        create_br(producer, cond_bb);
        producer.set_current_bb(cond_bb);
        // Cond logic
        let cond_res = self.continue_condition.produce_llvm_ir(producer);
        // XXX: Assumption: If the value is 0 the we go to the end block
        let cond = create_conditional_branch(
            producer,
            cond_res.expect("Conditional check expression must produce a value").into_int_value(),
            body_bb,
            end_bb
        );

        producer.set_current_bb(body_bb);
        // Body logic
        for stmt in &self.body {
            stmt.produce_llvm_ir(producer);
        }
        create_br(producer, cond_bb);
        producer.set_current_bb(end_bb);

        // Returns the condition code
        Some(cond)
    }
}

impl WriteWasm for LoopBucket {
    fn produce_wasm(&self, producer: &WASMProducer) -> Vec<String> {
        use code_producers::wasm_elements::wasm_code_generator::*;
        let mut instructions = vec![];
        if producer.needs_comments() {
            instructions.push(format!(";; loop bucket. Line {}", self.line)); //.to_string()
	}
        instructions.push(add_block());
        instructions.push(add_loop());
        let mut instructions_continue = self.continue_condition.produce_wasm(producer);
        instructions.append(&mut instructions_continue);
        instructions.push(call("$Fr_isTrue"));
        instructions.push(eqz32());
        instructions.push(br_if("1"));
        for ins in &self.body {
            let mut instructions_loop = ins.produce_wasm(producer);
            instructions.append(&mut instructions_loop);
        }
        instructions.push(br("0"));
        instructions.push(add_end());
        instructions.push(add_end());
        if producer.needs_comments() {
            instructions.push(";; end of loop bucket".to_string());
	}
        instructions
    }
}

impl WriteC for LoopBucket {
    fn produce_c(&self, producer: &CProducer, parallel: Option<bool>) -> (Vec<String>, String) {
        use c_code_generator::merge_code;
        let (continue_code, continue_result) = self.continue_condition.produce_c(producer, parallel);
        let continue_result = format!("Fr_isTrue({})", continue_result);
        let mut body = vec![];
        for instr in &self.body {
            let (mut instr_code, _) = instr.produce_c(producer, parallel);
            body.append(&mut instr_code);
        }
        body.append(&mut continue_code.clone());
        let while_loop = format!("while({}){{\n{}}}", continue_result, merge_code(body));
        let mut loop_c = continue_code;
        loop_c.push(while_loop);
        (loop_c, "".to_string())
    }
}
