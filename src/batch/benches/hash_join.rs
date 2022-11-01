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
use piestream_batch::executor::hash_join::HashJoinExecutor;
use piestream_batch::executor::test_utils::{gen_projected_data, MockExecutor};
use piestream_batch::executor::{BoxedExecutor, JoinType};
use piestream_common::catalog::schema_test_utils::field_n;
use piestream_common::hash;
use piestream_common::types::{DataType, ScalarImpl};
use piestream_expr::expr::build_from_prost;
use piestream_pb::data::data_type::TypeName;
use piestream_pb::expr::expr_node::RexNode;
use piestream_pb::expr::expr_node::Type::{
    ConstantValue as TConstValue, GreaterThan, InputRef, Modulus,
};
use piestream_pb::expr::{ConstantValue, ExprNode, FunctionCall, InputRefExpr};
use tikv_jemallocator::Jemalloc;
use utils::bench_join;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn create_hash_join_executor(
    join_type: JoinType,
    with_cond: bool,
    left_chunk_size: usize,
    left_chunk_num: usize,
    right_chunk_size: usize,
    right_chunk_num: usize,
) -> BoxedExecutor {
    let left_mod123 = {
        let input_ref = ExprNode {
            expr_type: InputRef as i32,
            return_type: Some(piestream_pb::data::DataType {
                type_name: TypeName::Int64 as i32,
                ..Default::default()
            }),
            rex_node: Some(RexNode::InputRef(InputRefExpr { column_idx: 0 })),
        };
        let literal123 = ExprNode {
            expr_type: TConstValue as i32,
            return_type: Some(piestream_pb::data::DataType {
                type_name: TypeName::Int64 as i32,
                ..Default::default()
            }),
            rex_node: Some(RexNode::Constant(ConstantValue {
                body: ScalarImpl::Int64(123).to_protobuf(),
            })),
        };
        ExprNode {
            expr_type: Modulus as i32,
            return_type: Some(piestream_pb::data::DataType {
                type_name: TypeName::Int64 as i32,
                ..Default::default()
            }),
            rex_node: Some(RexNode::FuncCall(FunctionCall {
                children: vec![input_ref, literal123],
            })),
        }
    };
    let right_mod456 = {
        let input_ref = ExprNode {
            expr_type: InputRef as i32,
            return_type: Some(piestream_pb::data::DataType {
                type_name: TypeName::Int64 as i32,
                ..Default::default()
            }),
            rex_node: Some(RexNode::InputRef(InputRefExpr { column_idx: 0 })),
        };
        let literal456 = ExprNode {
            expr_type: TConstValue as i32,
            return_type: Some(piestream_pb::data::DataType {
                type_name: TypeName::Int64 as i32,
                ..Default::default()
            }),
            rex_node: Some(RexNode::Constant(ConstantValue {
                body: ScalarImpl::Int64(456).to_protobuf(),
            })),
        };
        ExprNode {
            expr_type: Modulus as i32,
            return_type: Some(piestream_pb::data::DataType {
                type_name: TypeName::Int64 as i32,
                ..Default::default()
            }),
            rex_node: Some(RexNode::FuncCall(FunctionCall {
                children: vec![input_ref, literal456],
            })),
        }
    };
    let left_input = gen_projected_data(
        left_chunk_size,
        left_chunk_num,
        build_from_prost(&left_mod123).unwrap(),
    );
    let right_input = gen_projected_data(
        right_chunk_size,
        right_chunk_num,
        build_from_prost(&right_mod456).unwrap(),
    );

    let mut left_child = Box::new(MockExecutor::new(field_n::<1>(DataType::Int64)));
    left_input.into_iter().for_each(|c| left_child.add(c));

    let mut right_child = Box::new(MockExecutor::new(field_n::<1>(DataType::Int64)));
    right_input.into_iter().for_each(|c| right_child.add(c));

    let output_indices = match join_type {
        JoinType::LeftSemi | JoinType::LeftAnti => vec![0],
        JoinType::RightSemi | JoinType::RightAnti => vec![0],
        _ => vec![0, 1],
    };

    let cond = if with_cond {
        let left_input_ref = ExprNode {
            expr_type: InputRef as i32,
            return_type: Some(piestream_pb::data::DataType {
                type_name: TypeName::Int64 as i32,
                ..Default::default()
            }),
            rex_node: Some(RexNode::InputRef(InputRefExpr { column_idx: 0 })),
        };
        let literal100 = ExprNode {
            expr_type: TConstValue as i32,
            return_type: Some(piestream_pb::data::DataType {
                type_name: TypeName::Int64 as i32,
                ..Default::default()
            }),
            rex_node: Some(RexNode::Constant(ConstantValue {
                body: ScalarImpl::Int64(100).to_protobuf(),
            })),
        };
        Some(ExprNode {
            expr_type: GreaterThan as i32,
            return_type: Some(piestream_pb::data::DataType {
                type_name: TypeName::Int64 as i32,
                ..Default::default()
            }),
            rex_node: Some(RexNode::FuncCall(FunctionCall {
                children: vec![left_input_ref, literal100],
            })),
        })
    } else {
        None
    }
    .map(|expr| build_from_prost(&expr).unwrap());

    Box::new(HashJoinExecutor::<hash::Key64>::new(
        join_type,
        output_indices,
        left_child,
        right_child,
        vec![0],
        vec![0],
        vec![false],
        cond,
        "HashJoinExecutor".into(),
    ))
}

fn bench_hash_join(c: &mut Criterion) {
    let with_conds = vec![false, true];
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
        "HashJoinExecutor",
        with_conds,
        join_types,
        create_hash_join_executor,
    );
}

criterion_group!(benches, bench_hash_join);
criterion_main!(benches);
