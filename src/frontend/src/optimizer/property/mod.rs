// Copyright 2022 PieDb Data
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

//! Define all property of plan tree node, which actually represent property of the node's result.
//!
//! We have physical property [`Order`] and [`Distribution`] which is on batch or stream operator,
//! also, we have logical property which all [`PlanNode`][PlanNode] has.
//!
//! We have not give any common abstract trait for the property yet. They are not so much and we
//! don't need get a common behaviour now. we can treat them as different traits of the
//! [`PlanNode`][PlanNode] now and refactor them when our optimizer need more
//! (such as an optimizer based on the Volcano/Cascades model).
//!
//! [PlanNode]: super::plan_node::PlanNode
//! 定义规划树节点的所有属性，这些属性实际上代表了节点结果的属性
//! 我们有批处理或流操作符上的物理属性['Order']和['Distribution']，
//! 还有所有['PlanNode'][PlanNode]都有的逻辑属性
//! 对于这种性质，我们还没有给出任何共同的抽象特征。它们不是那么多，
//! 我们现在不需要有共同的行为。我们现在可以将它们视为['PlanNode'][PlanNode]的不同特性，
//! 并在优化器需要更多时重构它们(例如基于Volcano/Cascades模型的优化器)
//! 
pub(crate) mod order;
pub use order::*;
mod distribution;
pub use distribution::*;
