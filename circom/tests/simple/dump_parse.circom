// REQUIRES: circom
// RUN: rm -rf %t && mkdir %t && %circom --dump_parse -o %t %s | sed -n 's/.*Written successfully:.* \(.*\)/\1/p' | xargs cat | FileCheck %s --match-full-lines

pragma circom 2.0.0;

template HelloWorld() {
    signal input a;
    signal output b;

    b <== a;
}

component main = HelloWorld();
//CHECK-LABEL: ProgramArchive {
//CHECK-NEXT :     id_max: 8,
//CHECK-NEXT :     file_id_main: [[FD:[0-9]+]],
//CHECK-LABEL:     file_library: FileLibrary {
//CHECK-NEXT :       files: SimpleFiles {
//CHECK-NEXT :           files: [
//CHECK-NEXT :               SimpleFile {
//CHECK-NEXT :                   name: "<generated>",
//CHECK-NEXT :                   source: "",
//CHECK-NEXT :                   line_starts: [
//CHECK-NEXT :                       0,
//CHECK-NEXT :                   ],
//CHECK-NEXT :               },
//CHECK-NEXT :               SimpleFile {
//CHECK-NEXT :                   name: {{.*}}
//CHECK-NEXT :                   source: {{.*}}
//        COM:                      (--match-full-lines ensures this matches to the end to avoid match
//        COM:                      problems caused by 'source' containing the full content of this file)
//CHECK-NEXT :                   line_starts: [
//        COM:                      (skip the long list of "line_starts")
//CHECK-LABEL:     functions: {},
//CHECK-LABEL:     templates: {
//CHECK-NEXT :         "HelloWorld": TemplateData {
//CHECK-NEXT :             file_id: [[FD]],
//CHECK-NEXT :             name: "HelloWorld",
//CHECK-NEXT :             body: Block {
//CHECK-NEXT :                 meta: Meta {
//CHECK-NEXT :                     elem_id: [[[0-9]+]],
//CHECK-NEXT :                     start: [[[0-9]+]],
//CHECK-NEXT :                     end: [[[0-9]+]],
//CHECK-NEXT :                     location: [[[0-9]+]]..[[[0-9]+]],
//CHECK-NEXT :                     file_id: Some(
//CHECK-NEXT :                         [[FD]],
//CHECK-NEXT :                     ),
//CHECK-NEXT :                     component_inference: None,
//CHECK-NEXT :                     type_knowledge: TypeKnowledge {
//CHECK-NEXT :                         reduces_to: None,
//CHECK-NEXT :                     },
//CHECK-NEXT :                     memory_knowledge: MemoryKnowledge {
//CHECK-NEXT :                         concrete_dimensions: None,
//CHECK-NEXT :                         full_length: None,
//CHECK-NEXT :                         abstract_memory_address: None,
//CHECK-NEXT :                     },
//CHECK-NEXT :                 },
//CHECK-NEXT :                 stmts: [
//CHECK-NEXT :                     InitializationBlock {
//CHECK-NEXT :                         meta: Meta {
//CHECK-NEXT :                             elem_id: [[[0-9]+]],
//CHECK-NEXT :                             start: [[[0-9]+]],
//CHECK-NEXT :                             end: [[[0-9]+]],
//CHECK-NEXT :                             location: [[[0-9]+]]..[[[0-9]+]],
//CHECK-NEXT :                             file_id: Some(
//CHECK-NEXT :                                 [[FD]],
//CHECK-NEXT :                             ),
//CHECK-NEXT :                             component_inference: None,
//CHECK-NEXT :                             type_knowledge: TypeKnowledge {
//CHECK-NEXT :                                 reduces_to: None,
//CHECK-NEXT :                             },
//CHECK-NEXT :                             memory_knowledge: MemoryKnowledge {
//CHECK-NEXT :                                 concrete_dimensions: None,
//CHECK-NEXT :                                 full_length: None,
//CHECK-NEXT :                                 abstract_memory_address: None,
//CHECK-NEXT :                             },
//CHECK-NEXT :                         },
//CHECK-NEXT :                         xtype: Var,
//CHECK-NEXT :                         initializations: [],
//CHECK-NEXT :                     },
//CHECK-NEXT :                     InitializationBlock {
//CHECK-NEXT :                         meta: Meta {
//CHECK-NEXT :                             elem_id: [[[0-9]+]],
//CHECK-NEXT :                             start: [[[0-9]+]],
//CHECK-NEXT :                             end: [[[0-9]+]],
//CHECK-NEXT :                             location: [[[0-9]+]]..[[[0-9]+]],
//CHECK-NEXT :                             file_id: Some(
//CHECK-NEXT :                                 [[FD]],
//CHECK-NEXT :                             ),
//CHECK-NEXT :                             component_inference: None,
//CHECK-NEXT :                             type_knowledge: TypeKnowledge {
//CHECK-NEXT :                                 reduces_to: None,
//CHECK-NEXT :                             },
//CHECK-NEXT :                             memory_knowledge: MemoryKnowledge {
//CHECK-NEXT :                                 concrete_dimensions: None,
//CHECK-NEXT :                                 full_length: None,
//CHECK-NEXT :                                 abstract_memory_address: None,
//CHECK-NEXT :                             },
//CHECK-NEXT :                         },
//CHECK-NEXT :                         xtype: Component,
//CHECK-NEXT :                         initializations: [],
//CHECK-NEXT :                     },
//CHECK-NEXT :                     InitializationBlock {
//CHECK-NEXT :                         meta: Meta {
//CHECK-NEXT :                             elem_id: [[[0-9]+]],
//CHECK-NEXT :                             start: [[[0-9]+]],
//CHECK-NEXT :                             end: [[[0-9]+]],
//CHECK-NEXT :                             location: [[[0-9]+]]..[[[0-9]+]],
//CHECK-NEXT :                             file_id: Some(
//CHECK-NEXT :                                 [[FD]],
//CHECK-NEXT :                             ),
//CHECK-NEXT :                             component_inference: None,
//CHECK-NEXT :                             type_knowledge: TypeKnowledge {
//CHECK-NEXT :                                 reduces_to: None,
//CHECK-NEXT :                             },
//CHECK-NEXT :                             memory_knowledge: MemoryKnowledge {
//CHECK-NEXT :                                 concrete_dimensions: None,
//CHECK-NEXT :                                 full_length: None,
//CHECK-NEXT :                                 abstract_memory_address: None,
//CHECK-NEXT :                             },
//CHECK-NEXT :                         },
//CHECK-NEXT :                         xtype: Signal(
//CHECK-NEXT :                             Input,
//CHECK-NEXT :                             [],
//CHECK-NEXT :                         ),
//CHECK-NEXT :                         initializations: [
//CHECK-NEXT :                             Declaration {
//CHECK-NEXT :                                 meta: Meta {
//CHECK-NEXT :                                     elem_id: [[[0-9]+]],
//CHECK-NEXT :                                     start: [[[0-9]+]],
//CHECK-NEXT :                                     end: [[[0-9]+]],
//CHECK-NEXT :                                     location: [[[0-9]+]]..[[[0-9]+]],
//CHECK-NEXT :                                     file_id: Some(
//CHECK-NEXT :                                         [[FD]],
//CHECK-NEXT :                                     ),
//CHECK-NEXT :                                     component_inference: None,
//CHECK-NEXT :                                     type_knowledge: TypeKnowledge {
//CHECK-NEXT :                                         reduces_to: None,
//CHECK-NEXT :                                     },
//CHECK-NEXT :                                     memory_knowledge: MemoryKnowledge {
//CHECK-NEXT :                                         concrete_dimensions: None,
//CHECK-NEXT :                                         full_length: None,
//CHECK-NEXT :                                         abstract_memory_address: None,
//CHECK-NEXT :                                     },
//CHECK-NEXT :                                 },
//CHECK-NEXT :                                 xtype: Signal(
//CHECK-NEXT :                                     Input,
//CHECK-NEXT :                                     [],
//CHECK-NEXT :                                 ),
//CHECK-NEXT :                                 name: "a",
//CHECK-NEXT :                                 dimensions: [],
//CHECK-NEXT :                                 is_constant: true,
//CHECK-NEXT :                                 is_anonymous: false,
//CHECK-NEXT :                             },
//CHECK-NEXT :                         ],
//CHECK-NEXT :                     },
//CHECK-NEXT :                     InitializationBlock {
//CHECK-NEXT :                         meta: Meta {
//CHECK-NEXT :                             elem_id: [[[0-9]+]],
//CHECK-NEXT :                             start: [[[0-9]+]],
//CHECK-NEXT :                             end: [[[0-9]+]],
//CHECK-NEXT :                             location: [[[0-9]+]]..[[[0-9]+]],
//CHECK-NEXT :                             file_id: Some(
//CHECK-NEXT :                                 [[FD]],
//CHECK-NEXT :                             ),
//CHECK-NEXT :                             component_inference: None,
//CHECK-NEXT :                             type_knowledge: TypeKnowledge {
//CHECK-NEXT :                                 reduces_to: None,
//CHECK-NEXT :                             },
//CHECK-NEXT :                             memory_knowledge: MemoryKnowledge {
//CHECK-NEXT :                                 concrete_dimensions: None,
//CHECK-NEXT :                                 full_length: None,
//CHECK-NEXT :                                 abstract_memory_address: None,
//CHECK-NEXT :                             },
//CHECK-NEXT :                         },
//CHECK-NEXT :                         xtype: Signal(
//CHECK-NEXT :                             Output,
//CHECK-NEXT :                             [],
//CHECK-NEXT :                         ),
//CHECK-NEXT :                         initializations: [
//CHECK-NEXT :                             Declaration {
//CHECK-NEXT :                                 meta: Meta {
//CHECK-NEXT :                                     elem_id: [[[0-9]+]],
//CHECK-NEXT :                                     start: [[[0-9]+]],
//CHECK-NEXT :                                     end: [[[0-9]+]],
//CHECK-NEXT :                                     location: [[[0-9]+]]..[[[0-9]+]],
//CHECK-NEXT :                                     file_id: Some(
//CHECK-NEXT :                                         [[FD]],
//CHECK-NEXT :                                     ),
//CHECK-NEXT :                                     component_inference: None,
//CHECK-NEXT :                                     type_knowledge: TypeKnowledge {
//CHECK-NEXT :                                         reduces_to: None,
//CHECK-NEXT :                                     },
//CHECK-NEXT :                                     memory_knowledge: MemoryKnowledge {
//CHECK-NEXT :                                         concrete_dimensions: None,
//CHECK-NEXT :                                         full_length: None,
//CHECK-NEXT :                                         abstract_memory_address: None,
//CHECK-NEXT :                                     },
//CHECK-NEXT :                                 },
//CHECK-NEXT :                                 xtype: Signal(
//CHECK-NEXT :                                     Output,
//CHECK-NEXT :                                     [],
//CHECK-NEXT :                                 ),
//CHECK-NEXT :                                 name: "b",
//CHECK-NEXT :                                 dimensions: [],
//CHECK-NEXT :                                 is_constant: true,
//CHECK-NEXT :                                 is_anonymous: false,
//CHECK-NEXT :                             },
//CHECK-NEXT :                         ],
//CHECK-NEXT :                     },
//CHECK-NEXT :                     Substitution {
//CHECK-NEXT :                         meta: Meta {
//CHECK-NEXT :                             elem_id: [[[0-9]+]],
//CHECK-NEXT :                             start: [[[0-9]+]],
//CHECK-NEXT :                             end: [[[0-9]+]],
//CHECK-NEXT :                             location: [[[0-9]+]]..[[[0-9]+]],
//CHECK-NEXT :                             file_id: Some(
//CHECK-NEXT :                                 [[FD]],
//CHECK-NEXT :                             ),
//CHECK-NEXT :                             component_inference: None,
//CHECK-NEXT :                             type_knowledge: TypeKnowledge {
//CHECK-NEXT :                                 reduces_to: Some(
//CHECK-NEXT :                                     Signal,
//CHECK-NEXT :                                 ),
//CHECK-NEXT :                             },
//CHECK-NEXT :                             memory_knowledge: MemoryKnowledge {
//CHECK-NEXT :                                 concrete_dimensions: None,
//CHECK-NEXT :                                 full_length: None,
//CHECK-NEXT :                                 abstract_memory_address: None,
//CHECK-NEXT :                             },
//CHECK-NEXT :                         },
//CHECK-NEXT :                         var: "b",
//CHECK-NEXT :                         access: [],
//CHECK-NEXT :                         op: AssignConstraintSignal,
//CHECK-NEXT :                         rhe: Variable {
//CHECK-NEXT :                             meta: Meta {
//CHECK-NEXT :                                 elem_id: [[[0-9]+]],
//CHECK-NEXT :                                 start: [[[0-9]+]],
//CHECK-NEXT :                                 end: [[[0-9]+]],
//CHECK-NEXT :                                 location: [[[0-9]+]]..[[[0-9]+]],
//CHECK-NEXT :                                 file_id: Some(
//CHECK-NEXT :                                     [[FD]],
//CHECK-NEXT :                                 ),
//CHECK-NEXT :                                 component_inference: None,
//CHECK-NEXT :                                 type_knowledge: TypeKnowledge {
//CHECK-NEXT :                                     reduces_to: Some(
//CHECK-NEXT :                                         Signal,
//CHECK-NEXT :                                     ),
//CHECK-NEXT :                                 },
//CHECK-NEXT :                                 memory_knowledge: MemoryKnowledge {
//CHECK-NEXT :                                     concrete_dimensions: None,
//CHECK-NEXT :                                     full_length: None,
//CHECK-NEXT :                                     abstract_memory_address: None,
//CHECK-NEXT :                                 },
//CHECK-NEXT :                             },
//CHECK-NEXT :                             name: "a",
//CHECK-NEXT :                             access: [],
//CHECK-NEXT :                         },
//CHECK-NEXT :                     },
//CHECK-NEXT :                 ],
//CHECK-NEXT :             },
//CHECK-NEXT :             num_of_params: 0,
//CHECK-NEXT :             name_of_params: [],
//CHECK-NEXT :             param_location: [[[0-9]+]]..[[[0-9]+]],
//CHECK-NEXT :             input_wires: {
//CHECK-NEXT :                 "a": WireData {
//CHECK-NEXT :                     wire_type: Signal,
//CHECK-NEXT :                     dimension: 0,
//CHECK-NEXT :                     tag_info: {},
//CHECK-NEXT :                 },
//CHECK-NEXT :             },
//CHECK-NEXT :             output_wires: {
//CHECK-NEXT :                 "b": WireData {
//CHECK-NEXT :                     wire_type: Signal,
//CHECK-NEXT :                     dimension: 0,
//CHECK-NEXT :                     tag_info: {},
//CHECK-NEXT :                 },
//CHECK-NEXT :             },
//CHECK-NEXT :             is_parallel: false,
//CHECK-NEXT :             is_custom_gate: false,
//CHECK-NEXT :             is_extern_c: false,
//CHECK-NEXT :             input_declarations: [
//CHECK-NEXT :                 (
//CHECK-NEXT :                     "a",
//CHECK-NEXT :                     0,
//CHECK-NEXT :                 ),
//CHECK-NEXT :             ],
//CHECK-NEXT :             output_declarations: [
//CHECK-NEXT :                 (
//CHECK-NEXT :                     "b",
//CHECK-NEXT :                     0,
//CHECK-NEXT :                 ),
//CHECK-NEXT :             ],
//CHECK-NEXT :         },
//CHECK-NEXT :     },
//CHECK-NEXT :     buses: {},
//CHECK-NEXT :     function_keys: {},
//CHECK-NEXT :     template_keys: {
//CHECK-NEXT :         "HelloWorld",
//CHECK-NEXT :     },
//CHECK-NEXT :     bus_keys: {},
//CHECK-NEXT :     public_inputs: [],
//CHECK-NEXT :     initial_template_call: Call {
//CHECK-NEXT :         meta: Meta {
//CHECK-NEXT :             elem_id: [[[0-9]+]],
//CHECK-NEXT :             start: [[[0-9]+]],
//CHECK-NEXT :             end: [[[0-9]+]],
//CHECK-NEXT :             location: [[[0-9]+]]..[[[0-9]+]],
//CHECK-NEXT :             file_id: Some(
//CHECK-NEXT :                 [[FD]],
//CHECK-NEXT :             ),
//CHECK-NEXT :             component_inference: None,
//CHECK-NEXT :             type_knowledge: TypeKnowledge {
//CHECK-NEXT :                 reduces_to: None,
//CHECK-NEXT :             },
//CHECK-NEXT :             memory_knowledge: MemoryKnowledge {
//CHECK-NEXT :                 concrete_dimensions: None,
//CHECK-NEXT :                 full_length: None,
//CHECK-NEXT :                 abstract_memory_address: None,
//CHECK-NEXT :             },
//CHECK-NEXT :         },
//CHECK-NEXT :         id: "HelloWorld",
//CHECK-NEXT :         args: [],
//CHECK-NEXT :     },
//CHECK-NEXT :     custom_gates: false,
//CHECK-NEXT : }
