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

use fixedbitset::FixedBitSet;
use piestream_common::error::Result;

use crate::binder::BoundQuery;
use crate::optimizer::plan_node::{LogicalLimit, LogicalTopN};
use crate::optimizer::property::{Order, RequiredDist};
use crate::optimizer::PlanRoot;
use crate::planner::Planner;

pub const LIMIT_ALL_COUNT: usize = usize::MAX / 2;

    // dml时候会调用，
    // pub use select::BoundSelect;
    // pub use set_expr::BoundSetExpr;
    // pub use statement::BoundStatement;
    // pub use update::BoundUpdate;
    // pub use values::BoundValues;

    // BoundValues 是写入insert  
    // BoundSelect 执行计划  是否是批处理，还是流处理
    /// expressions表达式，<= ,>
    /// 
    /// 此函数目的是为了根据BoundQuery生成PlanRoot计划
    /// 有关的命令，create_mv、insert、relation、select、

impl Planner {
    /// Plan a [`BoundQuery`]. Need to bind before planning.
    /*
        insert..values模式
        insert into t5 values(7,77,'123')
        BoundQuery { 
            body: Values(BoundValues { 
                rows: [[7:Int32, 77:Int32, '哪':Varchar]], 
                //name:data_type 上面insert模式name为空
                schema: Schema { fields: [:Int32, :Int32, :Varchar] } 
            }), 
            order: [], 
            limit: None, 
            offset: None, 
            extra_order_exprs: [] //额外的排序表达式，是带表达式的排序写法， 例如order by id = 1,age;意思是只对id = 1的数据排序。
        }, 
     */
    pub fn plan_query(&mut self, query: BoundQuery) -> Result<PlanRoot> {
        //表达式长度
        let extra_order_exprs_len = query.extra_order_exprs.len();
        let out_names = query.schema().names();
        let mut plan = self.plan_set_expr(query.body, query.extra_order_exprs)?;
        //order\limlit特征关键字        
        /*
            当extra_order_exprs不为空时plan.schema除了select_item信息 还会包含extra_order_exprs信息
            plan.schema:Schema {
                fields: [
                    id:Int32,
                    age:Int32,
                    name:Varchar,
                    expr#3:Int32,
                ],
            }
        */
        let order = Order {
            //insert .. select order 模式
            field_order: query.order,
        };
        if query.limit.is_some() || query.offset.is_some() {
             //insert .. select  [offset] [limit] [order]模式
            let limit = query.limit.unwrap_or(LIMIT_ALL_COUNT);
            let offset = query.offset.unwrap_or_default();
            plan = if order.field_order.is_empty() {
                // Create a logical limit if with limit/offset but without order-by
                // 创建一个逻辑限制，如果有limit/offset，但没有order-by
                LogicalLimit::create(plan, limit, offset)
            } else {
                //topn非排序模式，不准确
                // Create a logical top-n if with limit/offset and order-by
                // 创建一个带有limit/offset和order-by的逻辑top-n if
                LogicalTopN::create(plan, limit, offset, order.clone())
            }
        }
        //设置分布式为单节点模式
        let dist = RequiredDist::single();
        //FixedBitSet长度等于字段长度
        let mut out_fields = FixedBitSet::with_capacity(plan.schema().len());
        out_fields.insert_range(..plan.schema().len() - extra_order_exprs_len);
        // 生成执行计划planroot
        let root = PlanRoot::new(plan, dist, order, out_fields, out_names);
        Ok(root)
    }
}
