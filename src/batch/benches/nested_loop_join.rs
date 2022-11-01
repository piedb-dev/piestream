// Copyright 2022 Piedb Data
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
pub mod utils;

use criterion::{criterion_group, criterion_main, Criterion};
use piestream_batch::executor::{BoxedExecutor, JoinType, NestedLoopJoinExecutor};
use piestream_common::types::{DataType, ScalarImpl};
use piestream_expr::expr::build_from_prost;
use piestream_pb::data::data_type::TypeName;
use piestream_pb::expr::expr_node::RexNode;
use piestream_pb::expr::expr_node::Type::{
    ConstantValue as TConstValue, Equal, InputRef, Modulus,
};
use piestream_pb::expr::{ConstantValue, ExprNode, FunctionCall, InputRefExpr};
use tikv_jemallocator::Jemalloc;
use utils::{bench_join, create_input};

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn create_nested_loop_join_executor(
    join_type: JoinType,
    _with_cond: bool,
    left_chunk_size: usize,
    left_chunk_num: usize,
    right_chunk_size: usize,
    right_chunk_num: usize,
) -> BoxedExecutor {
    let left_input = create_input(&[DataType::Int64], left_chunk_size, left_chunk_num);
    let right_input = create_input(&[DataType::Int64], right_chunk_size, right_chunk_num);

    // Expression: $1 % 2 == $2 % 3
    let join_expr = {
        let left_input_ref = ExprNode {
            expr_type: InputRef as i32,
            return_type: Some(piestream_pb::data::DataType {
                type_name: TypeName::Int64 as i32,
                ..Default::default()
            }),
            rex_node: Some(RexNode::InputRef(InputRefExpr { column_idx: 0 })),
        };

        let right_input_ref = ExprNode {
            expr_type: InputRef as i32,
            return_type: Some(piestream_pb::data::DataType {
                type_name: TypeName::Int64 as i32,
                ..Default::default()
            }),
            rex_node: Some(RexNode::InputRef(InputRefExpr { column_idx: 1 })),
        };

        let literal2 = ExprNode {
            expr_type: TConstValue as i32,
            return_type: Some(piestream_pb::data::DataType {
                type_name: TypeName::Int64 as i32,
                ..Default::default()
            }),
            rex_node: Some(RexNode::Constant(ConstantValue {
                body: ScalarImpl::Int64(2).to_protobuf(),
            })),
        };

        let literal3 = ExprNode {
            expr_type: TConstValue as i32,
            return_type: Some(piestream_pb::data::DataType {
                type_name: TypeName::Int64 as i32,
                ..Default::default()
            }),
            rex_node: Some(RexNode::Constant(ConstantValue {
                body: ScalarImpl::Int64(3).to_protobuf(),
            })),
        };

        // $1 % 2
        let left_mod2 = ExprNode {
            expr_type: Modulus as i32,
            return_type: Some(piestream_pb::data::DataType {
                type_name: TypeName::Int64 as i32,
                ..Default::default()
            }),
            rex_node: Some(RexNode::FuncCall(FunctionCall {
                children: vec![left_input_ref, literal2],
            })),
        };

        // $2 % 3
        let right_mod3 = ExprNode {
            expr_type: Modulus as i32,
            return_type: Some(piestream_pb::data::DataType {
                type_name: TypeName::Int64 as i32,
                ..Default::default()
            }),
            rex_node: Some(RexNode::FuncCall(FunctionCall {
                children: vec![right_input_ref, literal3],
            })),
        };

        // $1 % 2 == $2 % 3
        ExprNode {
            expr_type: Equal as i32,
            return_type: Some(piestream_pb::data::DataType {
                type_name: TypeName::Boolean as i32,
                ..Default::default()
            }),
            rex_node: Some(RexNode::FuncCall(FunctionCall {
                children: vec![left_mod2, right_mod3],
            })),
        }
    };

    let output_indices = match join_type {
        JoinType::LeftSemi | JoinType::LeftAnti => vec![0],
        JoinType::RightSemi | JoinType::RightAnti => vec![0],
        _ => vec![0, 1],
    };

    Box::new(NestedLoopJoinExecutor::new(
        build_from_prost(&join_expr).unwrap(),
        join_type,
        output_indices,
        left_input,
        right_input,
        "NestedLoopJoinExecutor".into(),
    ))
}

fn bench_nested_loop_join(c: &mut Criterion) {
    let with_conds = vec![false];
    let join_types = vec![
        JoinType::Inner,
        JoinType::LeftOuter,
        JoinType::LeftSemi,
        JoinType::LeftAnti,
        JoinType::RightOuter,
        JoinType::RightSemi,
        JoinType::RightAnti,
    ];
    bench_join(
        c,
        "NestedLoopJoinExecutor",
        with_conds,
        join_types,
        create_nested_loop_join_executor,
    );
}

criterion_group!(benches, bench_nested_loop_join);
criterion_main!(benches);
