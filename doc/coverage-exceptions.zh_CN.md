# 覆盖率阈值例外

crate 使用共享 `rs-ci` 阈值：函数覆盖率至少 95%，行覆盖率高于 90%，区域覆盖率高于
85%。本文件的审计结果使用 `cargo-llvm-cov 0.8.6`、Rust 1.94.0，在 2026-09-10 执行
`./coverage.sh json` 并启用全部 feature 生成。原始报告位于
`target/llvm-cov/coverage.json`，不提交到仓库。

本次运行后删除了三个原有例外，因为它们已经满足全部阈值：

- `src/multi_values/multi_values_identity.rs`（100% / 94.74% / 96.67%）。
- `src/value/value_identity.rs`（100% / 100% / 93.94%）。
- `src/value_missing.rs`（100% / 97.73% / 95.45%）。

其余条目经过同一份报告审计后继续保留。它们都是宽泛的公共 API 或协议边界，较低覆盖率
来自防御性分支、feature 专属分支或仅失败时执行的路径。完整 feature 集成测试和定向契约
测试继续作为可达行为的证据；为了执行不可能的分支而添加模拟测试会降低测试的代表性。

| 文件 | 代表性未覆盖区域 | 复现与替代证据 | 不缩小例外范围的原因 |
| --- | --- | --- | --- |
| `src/identity/json_identity.rs` | `hash_json_iterative` 与相等 visitor 退出（42-99、126-140） | `cargo test --all-features`；`identity::json_identity_tests` 覆盖深度、无序、预算和不匹配场景 | 未覆盖分支与迭代状态机耦合，拆分会复制实现。 |
| `src/multi_values/multi_values.rs` | `get_first_ref`、`get_slice` 和泛型 `set`/`add` 分支（186-214、914-970） | `cargo test --all-features`；`multi_values_*_tests` 覆盖所有存储类型与转换策略 | 宏生成的类型面是一个公共 API，逐分支例外会隐藏同一泛型实现。 |
| `src/named_multi_values.rs` | `serialize`、`deserialize` 和有界 writer 路径（413-468、248-298） | `cargo test --all-features`；`named_multi_values_tests` 与 wire 契约测试 | 路径同时依赖 `MultiValues` 和命名 envelope，按行拆分不能代表独立单元。 |
| `src/named_value.rs` | `serialize`、`deserialize` 和有界 writer 路径（363-408、231-281） | `cargo test --all-features`；`named_value_tests` 与有界 wire 测试 | 包装器分支取决于下游类型组合和协议 envelope。 |
| `src/value/value.rs` | 泛型 `get_or`、`to_or` 和转换分派（446-640） | `cargo test --all-features`；`value::*_tests` 覆盖全部支持的 feature 类型 | 剩余路径由 feature 矩阵生成，无法缩减为一个稳定公共场景。 |
| `src/value_container.rs` | `set`、`add` 和 `add_scalar`（497-773） | `cargo test --all-features`；`value_container_tests` 与 core feature 测试 | 标量/集合形状属于同一事务，拆分会让阈值配置跟随实现细节变化。 |
| `src/value_error.rs` | `PartialEq::eq` 和转换 `From` 实现（105-199） | `cargo test --all-features`；`value_error_tests` 覆盖公共错误构造和来源 | 失败变体由泛型转换上下文选择，缩小到行会很脆弱。 |
| `src/value_wire/value_wire_payload_v1.rs` | `to_json_writer_with_limits` 和 `deserialize`（255-360） | `cargo test --all-features`；payload golden、解码和错误测试 | 这些是协议边界分支，只在外部输入损坏或注入 writer 错误时发生。 |
| `src/value_wire/value_wire_ref_v1.rs` | `from_container` 和 `to_json_writer_with_limits`（102-218） | `cargo test --all-features`；借用 wire 和有界 writer 测试 | 分支共享借用序列化器，无法绕过所有权契约独立测试。 |
| `src/value_wire/value_wire_v1.rs` | `decode_json_slice_with_limits` 和 `to_json_writer_with_limits`（160-275） | `cargo test --all-features`；V1 golden、解码和限制测试 | 这些防御性协议分支需要损坏字节或注入 I/O；保留文件级范围便于审计。 |
| `src/wire.rs` | `deserialize_canonical`、vector visitor 和类型序列化器（90-206） | `cargo test --all-features`；wire golden 与严格解码测试 | wire 枚举是所有可选值类型的统一分派边界。 |
| `src/wire/internal/strict_string_map.rs` | `StrictStringMapVisitor::visit_map`（48-84） | `cargo test --all-features`；严格 string-map 与 wire 拒绝测试 | 拒绝路径需要损坏序列化 map，和严格 visitor 状态不可分离。 |

重新执行审计：

```text
COVERAGE_OPEN_HTML=0 ./coverage.sh json
```
